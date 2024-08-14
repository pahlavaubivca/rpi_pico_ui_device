#![no_std]
#![no_main]

pub mod lcd;
mod utils;
mod jobs;
mod error_types;

extern crate embedded_hal;
extern crate panic_halt;
extern crate rp2040_hal;
extern crate embedded_graphics;
extern crate embedded_graphics_core;
extern crate cortex_m;
extern crate defmt;
extern crate defmt_rtt;
extern crate heapless;
// extern crate panic_probe;

use core::any::{Any, TypeId};
use core::convert::TryInto;
use core::fmt::Debug;
use cortex_m::asm::delay;
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
use rp2040_hal::multicore::{Multicore, Stack};
use rp2040_hal::uart;
use rp2040_hal::uart::{DataBits, Error, StopBits, UartConfig};
use jobs::core0;
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
static mut CORE1_STACK: Stack<2048> = Stack::new();

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
    let mut sio = hal::Sio::new(pac.SIO);

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
    disp.set_orientation(&Orientation::LandscapeSwapped).unwrap();

    disp.set_offset(1, 2);
    disp.set_address_window(0, 0, 127, 127).unwrap();
    disp.clear(Rgb565::BLACK).unwrap();

    lcd_led.set_high().unwrap();
    led_pin.set_high().unwrap();

    let mut green_led_pin = pins.gpio2.into_push_pull_output();
    let mut blue1_led_pin = pins.gpio3.into_push_pull_output();
    let mut blue2_led_pin = pins.gpio4.into_push_pull_output();
    let mut red_led_pin = pins.gpio5.into_push_pull_output();

    let mut buzzer_pin = pins.gpio26.into_push_pull_output();

    green_led_pin.set_high().unwrap();
    blue1_led_pin.set_high().unwrap();
    blue2_led_pin.set_high().unwrap();
    red_led_pin.set_high().unwrap();
    buzzer_pin.set_high().unwrap();

    delay.delay_ms(1000u32);

    green_led_pin.set_low().unwrap();
    blue1_led_pin.set_low().unwrap();
    blue2_led_pin.set_low().unwrap();
    red_led_pin.set_low().unwrap();
    buzzer_pin.set_low().unwrap();

    let uart_pins = (
        pins.gpio16.into_function(),
        pins.gpio17.into_function(),
    );
    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(
                // 9600.Hz(),
                115_200.Hz(),
                //19_200.Hz(),
                DataBits::Seven,
                None,
                StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core0 = &mut cores[1];


    let mut down_button_pin = pins.gpio21.into_pull_up_input();
    let mut up_button_pin = pins.gpio19.into_pull_up_input();
    let mut left_button_pin = pins.gpio18.into_pull_up_input();
    let mut right_button_pin = pins.gpio22.into_pull_up_input();
    let mut ok_button_pin = pins.gpio20.into_pull_up_input();
    
    let _test = core0.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        jobs::core0(
            &mut uart,
            &mut down_button_pin,
            &mut up_button_pin,
            &mut left_button_pin,
            &mut right_button_pin,
            &mut ok_button_pin,
        );
    });

    jobs::core1(&mut disp);
}
