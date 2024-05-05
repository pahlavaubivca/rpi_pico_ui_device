#![no_std]
#![no_main]

pub mod lcd;

extern crate embedded_hal;
extern crate panic_halt;
extern crate rp2040_hal;
extern crate embedded_graphics;
extern crate embedded_graphics_core;
extern crate cortex_m;
extern crate defmt;
extern crate defmt_rtt;
// extern crate panic_probe;

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::RgbColor;
use embedded_hal::digital::OutputPin;
// Alias for our HAL crate
use rp2040_hal as hal;

use hal::pac;
// use rp2040_boot2;
use rp2040_hal::clocks::Clock;
use rp2040_hal::fugit::RateExtU32;
use lcd::lcd::{Orientation, ST7735};

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
    let _spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();

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


    // cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    // let mut delay_ns = cortex_m::delay::Delay::<hal::clocks::>::new();

    disp.init(&mut delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.set_offset(1, 2);
    disp.clear(Rgb565::BLACK).unwrap();


    let image_raw: ImageRawLE<Rgb565> =
        ImageRaw::new(include_bytes!("./assets/ferris.raw"), 86);

    let image: Image<_> = Image::new(&image_raw, Point::new(34, 8));

    image.draw(&mut disp).unwrap();

    lcd_led.set_high().unwrap();

    loop {
        defmt::info!("echo...");
        // continue;
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}

// End of file
