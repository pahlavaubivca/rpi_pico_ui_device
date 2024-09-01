use defmt::println;
use heapless::{String, Vec};

pub fn string_to_kv<const ILEN: usize, const OLEN: usize>(data: &String<ILEN>)
                                                          -> Result<Vec<(&str, &str), OLEN>, StringToKVError>
{
    let mut kv = Vec::new();
    let raw_split = data.split('&');
    println!("raw_split");
    let parts = data.split('&')
        .into_iter();
        // .collect::<Vec<&str, ILEN>>();
    println!("parts len");
    for part in parts {
        println!("part inside interator: {:?}", part);
        let key_value = part.split('=').collect::<Vec<&str, 2>>();
        kv.push((key_value[0], key_value[1])).expect("TODO: panic message");
    }
    Ok(kv)
}

pub enum StringToKVError {
    TooManyKeys,
    TooManyValues,
    TooManyPairs,
}