use cortex_m::delay::Delay;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::digital::OutputPin;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::gpio::Pins;
use lcd::lcd::{Orientation, ST7735};
use rp2040_hal as hal;
use hal::pac;
use rp2040_hal::spi::{SpiDevice, ValidSpiPinout};

pub mod lcd;
pub mod instruction;

// pub fn initialize_lcd<'a, DC, RST, D, PP>(
//     pac: &'a mut pac::Peripherals,
//     pins: &hal::gpio::Pins,
//     clocks: &hal::clocks::ClocksManager,
//     delay: &mut Delay
//     // delay: &mut impl DelayMs<u16>,
//     // mut pac: pac::Peripherals,
//     // pins: &mut Pins,
// ) -> ST7735<DC, RST, D, PP>
// where
//     DC: OutputPin,
//     RST: OutputPin,
//     D: SpiDevice,
//     PP: ValidSpiPinout<D>,
// {
//     let _spi_sclk = pins.gpio6.into_function::<hal::gpio::FunctionSpi>();
//     let _spi_mosi = pins.gpio7.into_function::<hal::gpio::FunctionSpi>();
//     // let _spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();
// 
//     let spi_pin_layout = (_spi_mosi, _spi_sclk);
// 
//     let spi = hal::Spi::<_, _, _, 8>::new(pac.SPI0, spi_pin_layout);
// 
//     let mut lcd_led = pins.gpio12.into_push_pull_output();
//     let dc = pins.gpio13.into_push_pull_output();
//     let rst = pins.gpio14.into_push_pull_output();
// 
//     // Exchange the uninitialised SPI driver for an initialised one
//     let spi = spi.init(
//         &mut pac.RESETS,
//         clocks.peripheral_clock.freq(),
//         16_000_000u32.Hz(),
//         &embedded_hal::spi::MODE_0,
//     );
// 
// 
//     let mut disp = ST7735::new(
//         spi,
//         dc,
//         Some(rst),
//         true, false, 128, 128);
// 
//     disp.init(&mut delay).unwrap();
//     disp.set_orientation(&Orientation::LandscapeSwapped).unwrap();
// 
//     disp.set_offset(1, 2);
//     disp.set_address_window(0, 0, 127, 127).unwrap();
//     disp.clear(Rgb565::BLACK).unwrap();
//     return disp;
// }