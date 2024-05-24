#![no_std]
#![no_main]

pub mod lcd;
mod utils;

extern crate embedded_hal;
extern crate panic_halt;
extern crate rp2040_hal;
extern crate embedded_graphics;
extern crate embedded_graphics_core;
extern crate cortex_m;
extern crate defmt;
extern crate defmt_rtt;
// extern crate panic_probe;

use core::any::{Any, TypeId};
use core::convert::TryInto;
use core::fmt::Debug;
use cortex_m::prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write};
// use core::fmt::Debug;
// use cortex_m::prelude::_embedded_hal_serial_Read;
use defmt::*;
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::{Point, Size};
use embedded_graphics_core::pixelcolor::{Rgb565, WebColors};
use embedded_graphics_core::prelude::RgbColor;
use embedded_graphics_core::primitives::Rectangle;
use embedded_hal::digital::OutputPin;
// Alias for our HAL crate
use rp2040_hal as hal;

use hal::pac;
// use rp2040_boot2;
use rp2040_hal::clocks::Clock;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::uart;
use rp2040_hal::uart::{DataBits, Error, StopBits, UartConfig};
use lcd::lcd::{Orientation, ST7735};
use utils::itoa::itoa;

// use panic_probe as _;
// use defmt::info;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Entry point to our bare-metal application.
///
/// The `#[rp2040_hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the RP2040 peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[rp2040_hal::entry]
// #[hal::entry]
fn main() -> ! {
    defmt::info!("Hello, world!");
    // info!("Hello, world!");
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();


    // Set up the watchdog driver - needed by the clock setup code
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
    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO25 as an output
    let mut led_pin = pins.gpio25.into_push_pull_output();

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_sclk = pins.gpio6.into_function::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio7.into_function::<hal::gpio::FunctionSpi>();
    // let _spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();

    let spi_pin_layout = (_spi_mosi, _spi_sclk);

    let spi = hal::Spi::<_, _, _, 8>::new(pac.SPI0, spi_pin_layout);

    let mut lcd_led = pins.gpio12.into_push_pull_output();
    let dc = pins.gpio13.into_push_pull_output();
    let rst = pins.gpio14.into_push_pull_output();

    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );


    let mut disp = ST7735::new(
        spi,
        dc,
        Some(rst),
        true, false, 128, 128);


    disp.init(&mut delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.set_offset(1, 2);
    disp.set_address_window(0, 0, 127, 127).unwrap();
    disp.clear(Rgb565::BLACK).unwrap();


    // let image_raw: ImageRawLE<Rgb565> =
    //     ImageRaw::new(include_bytes!("./assets/ferris.raw"), 86);
    //
    // let image: Image<_> = Image::new(&image_raw, Point::new(34, 8));
    //
    // image.draw(&mut disp).unwrap();

    lcd_led.set_high().unwrap();
    led_pin.set_high().unwrap();

    let mut counter = 0;
    let mut first_draw = true;


    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        // pins.gpio4.into_function(),
        pins.gpio16.into_function(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        // pins.gpio5.into_function(),
        pins.gpio17.into_function(),
    );
    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(19_200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();
    loop {
        let mut lines_to_display: [Option<&str>; 10] = [None; 10];
        if uart.uart_is_writable() {
            info!("uart is writable. {}",counter);
            // _ = uart.flush();
            
            uart.write_full_blocking(b"q\r\n");
            // let uart_write_result = uart.write('q' as u8).unwrap();
        }
        // if uart.uart_is_readable() {
        //     info!("UART is readable");
        //     let mut uart_buffer = [0u8; 1];
        //     let uart_read_result = uart.read_full_blocking(&mut uart_buffer);
        //     // let uart_read_result = uart.read();
        //     match uart_read_result {
        //         Ok(_) => {
        //             let uart_buffer_str = core::str::from_utf8(&uart_buffer).unwrap();
        //             info!("UART Read: {:?}", uart_buffer_str);
        //             lines_to_display[6] = Some("uart read ok");
        //         }
        //         Err(err) => {
        //             let mut message = "";
        //             // let err_map = err.map(|e| {
        //             //     message = match e
        //             //     {
        //             //         uart::ReadErrorType::Break => "UART Read: Break",
        //             //         uart::ReadErrorType::Overrun => "UART Read: Overrun",
        //             //         uart::ReadErrorType::Parity => "UART Read: Parity",
        //             //         uart::ReadErrorType::Framing => "UART Read: Framing"
        //             //     };
        //             // });
        // 
        //             let message = match err   {
        //                 uart::ReadErrorType::Break => "UART Read: Break",
        //                 uart::ReadErrorType::Overrun => "UART Read: Overrun",
        //                 uart::ReadErrorType::Parity => "UART Read: Parity",
        //                 uart::ReadErrorType::Framing => "UART Read: Framing"
        //             };
        //             lines_to_display[7] = Some(message);
        //         }
        //     }
        // } else {
        //     lines_to_display[8] = Some("UART serial not available");
        // }

        // ----- START draw on the screen ------
        // todo move to separate function
        // disp.clear(Rgb565::BLACK).unwrap();


        counter += 1;

        let counter_buffer = itoa(counter);
        let counter_str = core::str::from_utf8(&counter_buffer).unwrap();
        lines_to_display[1] = Some(&counter_str);
        lines_to_display[0] = Some("Hello, world!");
        let mut index = 0;


        for line in lines_to_display {
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
                ).draw(&mut disp).unwrap();

                _ = Line::new(
                    Point::new(2, offset_y + 2),
                    Point::new(125, offset_y + 2),
                )
                    .into_styled(PrimitiveStyle::with_stroke(Rgb565::BLUE, 1))
                    .draw(&mut disp).unwrap();
            }

            _ = Rectangle::new(
                Point::new(15, offset_y - 8),
                Size::new(113, 10),
            ).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(&mut disp).unwrap();

            if let Some(line) = line {
                // Clean up area for text


                // write text
                _ = Text::new(
                    line,
                    Point::new(15, offset_y),
                    MonoTextStyle::new(&FONT_6X12, Rgb565::RED),
                ).draw(&mut disp).unwrap();
            }
            index += 1;
        }

        first_draw = false;

        // ----- END draw on the screen ------
    }
}

// End of file
