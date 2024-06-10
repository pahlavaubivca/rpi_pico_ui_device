use cortex_m::delay::Delay;
use cortex_m::prelude::_embedded_hal_serial_Write;
use defmt::println;
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::{Point, Size};
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::RgbColor;
use embedded_graphics_core::primitives::Rectangle;
use embedded_hal::digital::{InputPin, OutputPin};
use heapless::String;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::{Clock, pac, Sio, uart};
use rp2040_hal::gpio::{FunctionSio, Pin, PullDown, PullType, PullUp, SioInput, SioOutput, ValidFunction};
use rp2040_hal::spi::{SpiDevice, ValidSpiPinout};
use rp2040_hal::uart::{DataBits, ReadErrorType, StopBits, UartConfig, UartDevice, UartPeripheral, ValidUartPinout};
use ::{rp2040_hal as hal, XTAL_FREQ_HZ};
use lcd::lcd::ST7735;
use utils::itoa::itoa;

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
    // let pins = hal::gpio::Pins::new(
    //     pac.IO_BANK0,
    //     pac.PADS_BANK0,
    //     sio.gpio_bank0,
    //     &mut pac.RESETS,
    // );
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
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut counter = 0;
    //button_pin.set_input_enable(true);
    // button_pin.set_high();
    loop {
        let is_down_button_pressed = down_button_pin.is_low().unwrap();
        let is_up_button_pressed = up_button_pin.is_low().unwrap();
        let is_left_button_pressed = left_button_pin.is_low().unwrap();
        let is_right_button_pressed = right_button_pin.is_low().unwrap();
        let is_ok_button_pressed = ok_button_pin.is_low().unwrap();
        println!("button down is pressed {:?}", is_down_button_pressed);
        println!("button up is pressed {:?}", is_up_button_pressed);
        println!("button left is pressed {:?}", is_left_button_pressed);
        println!("button right is pressed {:?}", is_right_button_pressed);
        println!("button ok is pressed {:?}", is_ok_button_pressed);

        if uart.uart_is_writable() {
            if is_down_button_pressed {
                _ = uart.flush();
                let uart_write_result = uart.write('d' as u8).unwrap();
            } else if is_up_button_pressed {
                _ = uart.flush();
                let uart_write_result = uart.write('u' as u8).unwrap();
            }
            // uart.write_full_blocking(b"Hello, world! from core1");
            // let uart_write_result = uart.write('q' as u8).unwrap();
        }


        let mut lines_to_send: [Option<(&str, bool)>; 10] = [None; 10];
        lines_to_send[0] = Some(("Hello, world! from core1", false));
        if uart.uart_is_readable() {
            println!("UART is readable");
            let mut len_buffer = [0u8; 4];
            let len_buf_result = uart.read_full_blocking(&mut len_buffer);
            if len_buf_result.is_ok() {
                let len = u32::from_be_bytes(len_buffer);
                println!("Received len: {:?}", len);
                let mut buffer = [0u8; 32];
                let mut index = 0;
                let mut text_buffer: String<2048> = String::new();
                while len > index * 32 {
                    println!("Reading: {:?}", index);
                    buffer = [0u8; 32];
                    let read_result = uart.read_full_blocking(&mut buffer);
                    match read_result {
                        Ok(_) => {
                            let uart_buffer_str = core::str::from_utf8(&buffer).unwrap();
                            text_buffer.push_str(uart_buffer_str).unwrap();
                            // let result = concat_strs_simple(str, uart_buffer_str);
                            // str = core::str::from_utf8(&result).unwrap();

                            println!("Received str: {:?}", uart_buffer_str);
                            index += 1;
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
                _ = uart.flush();
                let lines_to_send = split_lines(text_buffer.as_str())
                    .map(|line| Some(line));
            }
        }
        println!("Sending: {:?}", lines_to_send.len());
        // lines_to_send[0] = Some(("Hello, world! from core1", false));
        // let counter_buffer = itoa(counter);
        // let counter_str = core::str::from_utf8(&counter_buffer).unwrap();
        // lines_to_send[1] = Some(&counter_str);
        //let some_str = "Hello, world! from core0 for core 1 to read";
        // println!("Sending: {}", some_str);
        //sio.fifo.write_blocking(some_str.as_ptr() as u32);

        // sio.fifo.write(&lines_to_send as *const _ as u32);

        sio.fifo.write(&lines_to_send as *const _ as u32);
        // delay.delay_ms(100u32);
        counter += 1;
    }
}

/// Responsible for drawing on screen
pub fn core1<DC, RST, D, PP>(display: &mut ST7735<DC, RST, D, PP>) -> ! where
    DC: OutputPin,
    RST: OutputPin,
    D: SpiDevice,
    PP: ValidSpiPinout<D>
{
    println!("Hello, world! from core1");
    let mut _sio = unsafe { pac::Peripherals::steal() }.SIO;
    let mut sio = Sio::new(_sio);
    let mut first_draw = true;
    loop {
        //  println!("core1 loop");
        let lines_ptr = sio.fifo.read_blocking() as *const [Option<&str>; 10];
        // println!("Received: {:?}", lines_ptr);
        let lines = unsafe { &*lines_ptr };

        // println!("Received: {:?}", lines);

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
                    line,
                    Point::new(15, offset_y),
                    MonoTextStyle::new(&FONT_6X12, Rgb565::RED),
                ).draw(display).unwrap();
            }
            index += 1;
        }
        first_draw = false;
        // if let Some(word) = lines {
        //     println!("Received: {}", word);
        //     // delay.delay_ms(word);
        //     // led_pin.toggle().unwrap();
        //     // sio.fifo.write_blocking(CORE1_TASK_COMPLETE);
        // };
    }
}

fn concat_strs_simple(a: &str, b: &str) -> [u8; 32] {
    let mut buffer = [0u8; 32]; // Buffer size needs to be sufficient
    let bytes_a = a.as_bytes();
    let bytes_b = b.as_bytes();

    buffer[..bytes_a.len()].copy_from_slice(bytes_a);
    buffer[bytes_a.len()..bytes_a.len() + bytes_b.len()].copy_from_slice(bytes_b);

    buffer
}

fn split_lines(input: &str) -> [&str; 10] {
    // Use a static array to store the references to substrings
    const MAX_LINES: usize = 10; // Adjust this as needed
    let mut result: [&str; MAX_LINES] = [""; MAX_LINES];
    let mut count = 0;
    let split_res = input.split('\n');
    for line in split_res {
        if count < MAX_LINES {
            result[count] = line.trim();
            count += 1;
        } else {
            break;
        }
    }

    result
}