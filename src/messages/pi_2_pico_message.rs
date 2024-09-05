use core::convert::{TryFrom, TryInto};
use defmt::{error, Format, println};
use heapless::{String, Vec};
use utils::string_to_kv::{string_to_kv, StringToKVError};

pub struct Pi2PicoMessage {
    pub cursor_index: Option<isize>,
    pub ip_and_battery: Option<(String<15>, String<3>)>, //IP/battery %
    pub title_and_paginator: Option<(String<15>, String<5>)>, //first header title, second page/total pages
    pub data_lines: Option<Vec<Option<String<20>>, 8>>,
}



impl TryFrom<&String<2048>> for Pi2PicoMessage {
    type Error = Pi2PicoMessageError;

    fn try_from(value: &String<2048>) -> Result<Self, Self::Error> {
        let mut pi2_pico_message = Pi2PicoMessage {
            cursor_index: None,
            ip_and_battery: None,
            title_and_paginator: None,
            data_lines: None,
        };
        match string_to_kv::<2048, 10>(&value) {
            Ok(kv) => {
                for x in kv {
                    match x {
                        ("cursor_index", cursor_index) => {
                            pi2_pico_message.cursor_index = Some(cursor_index.parse::<isize>().unwrap());
                        }
                        ("ip_and_battery", ip_and_battery) => {
                            let mut ip_and_battery_iter = ip_and_battery.split("/");
                            let ip = ip_and_battery_iter.next().unwrap();
                            let battery = ip_and_battery_iter.next().unwrap();
                            pi2_pico_message.ip_and_battery = Some((String::from(ip), String::from(battery)));
                        }
                        ("title_and_paginator", title_and_paginator) => {
                            let mut title_and_paginator_iter = title_and_paginator.split("/");
                            let title = title_and_paginator_iter.next().unwrap();
                            let paginator = title_and_paginator_iter.next().unwrap();
                            pi2_pico_message.title_and_paginator = Some((String::from(title), String::from(paginator)));
                        }
                        ("data_lines", data_lines) => {
                            let data_lines_iter = data_lines.split(",");
                            let data_lines = data_lines_iter
                                .map(|x| Some(String::from(x)))
                                .collect::<Vec<Option<String<20>>, 8>>();
                            pi2_pico_message.data_lines = Some(data_lines);
                        }

                        _ => {}
                    }
                }
            }
            Err(err) => {
                error!("Error: {:?}", err);
                return Err(Pi2PicoMessageError::ParseError);
            }
        }
        Err(Pi2PicoMessageError::StringMismatch)
    }
}


#[derive(Format)]
pub enum Pi2PicoMessageError{
    StringMismatch,
    ParseError,
}