/// Convert and integer to byte array of ASCII characters
/// int32 have max 10 digits, 1 for sign and 1 for null terminator, so return array on 12 u8 bytes
pub fn itoa(num: i32) -> [u8; 12] {
    let mut buffer = [b' '; 12];
    let mut i = 10; // Start filling buffer from the end. 10 is total possible digits in i32
    let mut num = num;
    // if number 0 - just return one byte array with '0'
    if num == 0 {
        buffer[i] = b'0';
        return buffer;
    }

    let is_negative = num < 0;
    if is_negative {
        num = -num; // if negative - convert to positive
    }

    while num > 0 {
        buffer[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i -= 1;
    }

    if is_negative {
        i -= 1;
        buffer[i] = b'-';
    }

    // Shift the buffer to the left to remove leading spaces
    let mut result = [b' '; 12];
    let mut j = 0;
    while i < 12 {
        result[j] = buffer[i];
        i += 1;
        j += 1;
    }

    result
}