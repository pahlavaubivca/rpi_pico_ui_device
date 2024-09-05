use core::convert::TryFrom;
use defmt::println;
use heapless::String;
use utils::string_to_kv::string_to_kv;

pub struct Pi2PicoTest {
    pub kc: String<1>,
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
                            let asdf = String::from(kc);
                            return Ok(Pi2PicoTest {
                                kc: asdf,
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
            kc: String::from(""), //default
        })
    }
}