use core::ops::Not;
use defmt::{Format, println};
use heapless::{String, Vec};

pub fn string_to_kv<const ILEN: usize, const OLEN: usize>(data: &String<ILEN>)
                                                          -> Result<Vec<(&str, &str), OLEN>, StringToKVError>
{
    if data.contains("=").not() {
        return Err(StringToKVError::NotAnKVString);
    }

    let mut kv = Vec::new();
    let parts = data.split('&')
        .into_iter();
    for part in parts {
        let key_value = part.split('=').collect::<Vec<&str, 2>>();
        kv.push((key_value[0], key_value[1])).expect("kv push panic message");
    }
    Ok(kv)
}

#[derive(Format)]
pub enum StringToKVError {
    NotAnKVString
}