use core::convert::Infallible;
use defmt::Formatter;
use rp2040_hal::uart::UartPeripheral;
// 
// impl defmt::Format for nb::Error<Infallible>{
//     fn format(&self, fmt: Formatter) {
//         todo!()
//     }
// }