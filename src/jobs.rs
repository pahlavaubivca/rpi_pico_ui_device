use core::convert::{Infallible, TryFrom};
use core::fmt::Write;
use core::ops::Deref;
use core::slice;
use core::str::Utf8Error;
use cortex_m::asm::delay;
use cortex_m::delay::Delay;

use cortex_m::prelude::_embedded_hal_serial_Write;
use defmt::{debug, error, Format, Formatter, info, println};
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::{Point, Size};
use embedded_graphics_core::pixelcolor::raw::ToBytes;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::RgbColor;
use embedded_graphics_core::primitives::Rectangle;
use embedded_hal::digital::{InputPin, OutputPin};
use heapless::{String, Vec};

use rp2040_hal::{Clock, pac, Sio, Timer, uart};
use rp2040_hal::gpio::{Error, FunctionSio, Pin, PullUp, SioInput, ValidFunction};
use rp2040_hal::spi::{SpiDevice, ValidSpiPinout};
use rp2040_hal::uart::{DataBits, ReadErrorType, StopBits, UartConfig, UartDevice, UartPeripheral, ValidUartPinout};
use ::{rp2040_hal as hal, XTAL_FREQ_HZ};
use lcd::lcd::ST7735;
use messages::pi_2_pico_message::Pi2PicoMessage;
use messages::pi_2_pico_test::Pi2PicoTest;
use utils::itoa::itoa;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyboardCodes {
    Up,
    Down,
    Left,
    Right,
    Ok,
}
impl KeyboardCodes {
    pub fn as_u8(&self) -> u8 {
        match self {
            KeyboardCodes::Up => 'u' as u8,
            KeyboardCodes::Down => 'd' as u8,
            KeyboardCodes::Left => 'l' as u8,
            KeyboardCodes::Right => 'r' as u8,
            KeyboardCodes::Ok => 'o' as u8,
        }
    }

    //obsolete
    pub fn as_char(&self) -> char {
        match self {
            KeyboardCodes::Up => 'u',
            KeyboardCodes::Down => 'd',
            KeyboardCodes::Left => 'l',
            KeyboardCodes::Right => 'r',
            KeyboardCodes::Ok => 'o',
        }
    }
}

pub struct Pico2PiMessage {
    pub wh: Option<[i32; 2]>,
    //todo: add multi-key press
    pub keyboard_codes: Option<KeyboardCodes>,
}


//todo read about ! mark as return type
/// Core responsible for handling keyboard input, uart IO
pub fn core0<
    D: UartDevice,
    P: ValidUartPinout<D>,
    ID: ValidFunction<FunctionSio<SioInput>>,
    IU: ValidFunction<FunctionSio<SioInput>>,
    IL: ValidFunction<FunctionSio<SioInput>>,
    IR: ValidFunction<FunctionSio<SioInput>>,
    IOK: ValidFunction<FunctionSio<SioInput>>,
>(
    uart: &mut UartPeripheral<rp2040_hal::uart::Enabled, D, P>,
    down_button_pin: &mut Pin<ID, FunctionSio<SioInput>, PullUp>,
    up_button_pin: &mut Pin<IU, FunctionSio<SioInput>, PullUp>,
    left_button_pin: &mut Pin<IL, FunctionSio<SioInput>, PullUp>,
    right_button_pin: &mut Pin<IR, FunctionSio<SioInput>, PullUp>,
    ok_button_pin: &mut Pin<IOK, FunctionSio<SioInput>, PullUp>,
) -> !
{
    println!("Hello, world! from core0");
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = hal::Sio::new(pac.SIO);
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut counter = 0;
    //button_pin.set_input_enable(true);
    // button_pin.set_high();
    let mut full_message: String<100> = String::new();


    let mut timestamp = timer.get_counter().ticks();

    let mut keycode_to_send_option: Option<KeyboardCodes> = None;
    let mut keydown_code_option: Option<KeyboardCodes> = None;

    let mut keydown_timestamp = timer.get_counter().ticks();

    // uart.enable_tx_interrupt();
    let mut text_buffer: String<2048> = String::new();

    // let mut lines_to_send: [Option<(&str, bool)>; 10] = [None; 10];
    // lines_to_send[5] = Some(("hw! core1", false));

    let mut ks: String<50> = String::new();
    let mut lines_so_send: [Option<(String<50>, bool)>; 10] = Default::default();
    lines_so_send[5] = Some((String::from("hw! core1"), false));
    loop {
        let general_timer = timer.get_counter().ticks();
        if down_button_pin.is_low().unwrap() {
            keydown_code_option = Some(KeyboardCodes::Down);
        } else if up_button_pin.is_low().unwrap() {
            keydown_code_option = Some(KeyboardCodes::Up);
        } else if left_button_pin.is_low().unwrap() {
            keydown_code_option = Some(KeyboardCodes::Left);
        } else if right_button_pin.is_low().unwrap() {
            keydown_code_option = Some(KeyboardCodes::Right);
        } else if ok_button_pin.is_low().unwrap() {
            keydown_code_option = Some(KeyboardCodes::Ok);
        } else {
            keydown_timestamp = 0u64;
            keydown_code_option = None;
        }

        if let Some(ckc) = keydown_code_option {
            if keydown_timestamp == 0 {
                keydown_timestamp = general_timer;
            }
            keycode_to_send_option = Some(ckc);
        }

        if uart.uart_is_readable() {
            let mut buffer = [0u8; 1];
            let read_result = uart.read_full_blocking(&mut buffer);
            match read_result {
                Ok(_) => {
                    match core::str::from_utf8(&buffer) {
                        Ok(uart_buffer_str) => {
                            text_buffer.push_str(uart_buffer_str).unwrap();
                            if text_buffer.contains("\r\n") {
                                println!("Buffer contains \\r\\n");
                                text_buffer = String::from(text_buffer.trim_end_matches("\r\n"));

                                match Pi2PicoMessage::try_from(&text_buffer) {
                                    Ok(message) => {
                                        //todo push title and paginator to last line
                                        //todo push ip and battery to first line
                                        //todo push data lines to lines 2-9
                                        //todo push cursor index to line as bool
                                    }
                                    Err(err) => {
                                        error!("Error reading Pi2PicoMessage: {:?}", err);
                                    }
                                }

                                let pi_2_pico_test = Pi2PicoTest::try_from(&text_buffer);
                                println!("Pi2PicoTest try_from done"); //::try_from(text_buffer);
                                match pi_2_pico_test {
                                    Ok(val) => {
                                        println!("Pi2PicoTest: match Ok");


                                        lines_so_send[1] = Some((String::from(val.kc.as_str()), false));
                                    }
                                    Err(_) => {}
                                }
                                text_buffer.clear();
                                uart.flush();
                            }
                        }
                        Err(err) => {
                            println!("Error reading core::str::from_utf8(&buffer)");
                        }
                    }
                }
                Err(err) => {
                    let message = match err {
                        uart::ReadErrorType::Break => "UART Read: Break",
                        uart::ReadErrorType::Overrun => "UART Read: Overrun",
                        uart::ReadErrorType::Parity => "UART Read: Parity",
                        uart::ReadErrorType::Framing => "UART Read: Framing"
                    };
                    println!("Error reading {:?}", message);
                }
            }
        }

        //run this block every 250ms
        if general_timer - timestamp > 250_000 {
            if uart.uart_is_writable() {
                // println!("keycode_to_send_option: {:?}", keycode.as_u8());
                let mut is_keys_the_same = false;
                if keydown_code_option.is_some() {
                    // is_keys_the_same = keydown_code_option.eq(&Some(*keycode));
                    is_keys_the_same = keydown_code_option.eq(&keycode_to_send_option);
                    // println!("is_keys_the_same {:?}", is_keys_the_same);
                }

                let mut kd_ms = 0;
                if keydown_timestamp > 0 {
                    kd_ms = (general_timer - keydown_timestamp) / 1_000;
                    // println!("keydown ms: {:?}", kd_ms);
                }
                //todo: maybe move logic with send message after 1 sec to state machine?
                let keyup_or_keypress_more_than_sec = (
                    keydown_code_option.is_none() ||
                        is_keys_the_same && kd_ms > 1000
                );

                if keycode_to_send_option.is_some() && keyup_or_keypress_more_than_sec
                {
                    let keycode = keycode_to_send_option.unwrap();
                    let mut message: String<50> = String::new();
                    message.push_str("&wh=").unwrap();
                    message.push_str("128,128").unwrap();
                    message.push_str("&kc=").unwrap();
                    message.push(keycode.as_char()).unwrap();
                    message.push_str("&keypressms=").unwrap();


                    //todo: remove this. its only for button debug
                    let mut message_to_screen: String<50> = String::from("kc: ");
                    message_to_screen.push(keycode.as_char()).unwrap();
                    lines_so_send[2] = Some((message_to_screen, false));
                    //

                    let kd_str_slice: String<5> = String::from(kd_ms);
                    message.push_str(kd_str_slice.as_str()).unwrap();

                    keycode_to_send_option = None;

                    let message_len = message.len() as u8;
                    let message_len_string: String<4> = String::from(message_len);
                    full_message = String::new();
                    full_message.push_str("len=").unwrap();
                    full_message.push_str(message_len_string.as_str()).unwrap();
                    full_message.push_str(message.as_str()).unwrap();
                    full_message.push_str("\r\n").unwrap();
                    println!("Message to send: {:?}", full_message.as_str());
                    for i in 0..full_message.len() {
                        // only write char and write blocking works as expected for now
                        let byte_to_send = full_message.as_bytes()[i];
                        uart.write_char(byte_to_send as char).map_err(|_| {
                            println!("Error writing to UART char {:?}", byte_to_send);
                        }).unwrap();
                        // uart.write_full_blocking(&[byte_to_send]);
                    }
                }
            }
            timestamp = general_timer;
        }

        sio.fifo.write(&lines_so_send as *const _ as u32);
    }
}

/// Responsible for drawing on screen
pub fn core1<DC, RST, D, PP>(display: &mut ST7735<DC, RST, D, PP>) -> !
where
    DC: OutputPin,
    RST: OutputPin,
    D: SpiDevice,
    PP: ValidSpiPinout<D>,
{
    println!("Hello, world! from core1");
    let mut _sio = unsafe { pac::Peripherals::steal() }.SIO;
    let mut sio = Sio::new(_sio);
    let mut first_draw = true;
    loop {
        let lines_ptr = sio.fifo.read_blocking() as *const [Option<(String<50>, bool)>; 10];
        let lines = unsafe { &*lines_ptr };

        let mut index = 0;
        for line in lines {
            let offset_y = (12 * (index + 1)) + 1;
            // I did not find a way to make something like - `to_str(num:i32) -> str`
            let num_line_buffer = itoa(index + 1);
            let num_line_str = core::str::from_utf8(&num_line_buffer).unwrap();

            // draw line and line number when first draw
            if first_draw {
                _ = Text::new(
                    num_line_str,
                    Point::new(0, offset_y),
                    MonoTextStyle::new(&FONT_6X12, Rgb565::WHITE),
                ).draw(display).unwrap();

                _ = Line::new(
                    Point::new(2, offset_y + 2),
                    Point::new(125, offset_y + 2),
                )
                    .into_styled(PrimitiveStyle::with_stroke(Rgb565::BLUE, 1))
                    .draw(display).unwrap();
            }

            _ = Rectangle::new(
                Point::new(15, offset_y - 8),
                Size::new(113, 10),
            ).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(display).unwrap();

            if let Some(line) = line {
                // Clean up area for text
                // write text
                _ = Text::new(
                    line.0.as_str(),
                    Point::new(15, offset_y),
                    MonoTextStyle::new(&FONT_6X12, Rgb565::RED),
                ).draw(display).unwrap();
            }
            index += 1;
        }
        first_draw = false;
    }
}

fn concat_str_simple(a: &str, b: &str) -> [u8; 32] {
    let mut buffer = [0u8; 32]; // Buffer size needs to be sufficient
    let bytes_a = a.as_bytes();
    let bytes_b = b.as_bytes();

    buffer[..bytes_a.len()].copy_from_slice(bytes_a);
    buffer[bytes_a.len()..bytes_a.len() + bytes_b.len()].copy_from_slice(bytes_b);

    buffer
}
