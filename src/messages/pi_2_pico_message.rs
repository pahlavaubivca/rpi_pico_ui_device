use core::convert::{TryFrom, TryInto};
use defmt::println;
use heapless::{String, Vec};
use utils::string_to_kv::{string_to_kv, StringToKVError};

pub struct Pi2PicoMessage {
    pub cursor_index: Option<isize>,
    pub ip_and_battery: Option<([[u8; 3]; 4], [u8; 3])>, //IP/battery %
    pub title_and_paginator: Option<([u8; 15], [u8; 5])>, //first header title, second page/total pages
    pub data_lines: Option<[Option<[u8; 20]>; 8]>,
}
pub struct Pi2PicoTest {
    pub kc: u8,
}
impl TryFrom<&String<2048>> for Pi2PicoTest {
    type Error = ();

    fn try_from(value: &String<2048>) -> Result<Self, Self::Error> {
        println!("try_from on convert string to kv.");
        match string_to_kv::<2048, 10>(&value) {
            Ok(kv) => {
                println!("convert string to kv success");
                for x in kv {
                    match x {
                        ("kc", kc) => {
                            println!("kc: {:?}", kc);
                            return Ok(Pi2PicoTest {
                                kc: kc.parse::<u8>().unwrap(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                println!("try_from error on convert string to kv.");
                // println!("Error: {:?}", err);
            }
        }
        Ok(Pi2PicoTest {
            kc: 0,
        })
    }
}
impl TryFrom<String<2048>> for Pi2PicoMessage {
    type Error = ();

    fn try_from(value: String<2048>) -> Result<Self, Self::Error> {
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
                            let mut ip_and_battery = [[0; 3]; 4];
                            let mut battery = [0; 3];
                            let mut ip_and_battery_iter = ip_and_battery.split_at_mut(3);
                            // for (i, x) in ip_and_battery_iter.0.iter_mut().enumerate() {
                            //     *x = ip_and_battery[i].try_into().unwrap();
                            // }
                            // for (i, x) in ip_and_battery_iter.1.iter_mut().enumerate() {
                            //     *x = ip_and_battery[i].try_into().unwrap();
                            // }
                            pi2_pico_message.ip_and_battery = Some((ip_and_battery, battery));
                        }
                        ("title_and_paginator", title_and_paginator) => {
                            let mut title_and_paginator = [0; 15];
                            let mut paginator = [0; 5];
                            let mut title_and_paginator_iter = title_and_paginator.split_at_mut(15);
                            // for (i, x) in title_and_paginator_iter.0.iter_mut().enumerate() {
                            //     *x = title_and_paginator[i].try_into().unwrap();
                            // }
                            // for (i, x) in title_and_paginator_iter.1.iter_mut().enumerate() {
                            //     *x = title_and_paginator[i].try_into().unwrap();
                            // }
                            pi2_pico_message.title_and_paginator = Some((title_and_paginator, paginator));
                        }
                        ("data_lines", data_lines) => {
                            let mut data_lines = [0; 20];
                            let mut data_lines_iter = data_lines.split_at_mut(20);
                            // for (i, x) in data_lines_iter.0.iter_mut().enumerate() {
                            //     *x = data_lines[i].try_into().unwrap();
                            // }
                            // for (i, x) in data_lines_iter.1.iter_mut().enumerate() {
                            //     *x = data_lines[i].try_into().unwrap();
                            // }
                            pi2_pico_message.data_lines = Some([Some(data_lines); 8]);
                        }

                        _ => {}
                    }
                }
            }
            Err(err) => {
                // println!("Error: {:?}", err);
            }
        }
        Ok(pi2_pico_message)
    }
}