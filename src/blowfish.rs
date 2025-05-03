const checksum_table: [char; 16] = [
    'f', 'X', '1', 'p', 'G', 't', 'd', 'S', '5', 'C', 'A', 'P', '4', '_', 'V', 'L',
];

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use physis::blowfish::Blowfish;
use windows::Win32::System::SystemInformation::*;

fn calculate_checksum(key: u32) -> char {
    let value = key & 0x000F0000;
    checksum_table[(value >> 16) as usize]
}

fn calculate_blowfish_key() -> u32 {
    let tick_count;
    unsafe {
        tick_count = GetTickCount();
    }

    let ticks = tick_count & 0xFFFFFFFFu32;
    ticks & 0xFFFF0000u32
}

fn initialize_blowfish() -> Blowfish {
    let buffer = format!("{:08x}", calculate_blowfish_key());
    Blowfish::new(buffer.as_bytes())
}

pub fn decrypt_arguments(commandline: &str) -> String {
    let index = commandline.find("sqex").unwrap();

    let base64 = &commandline[index + 8..commandline.len() - 6];

    let blowfish = initialize_blowfish();
    let decoded = URL_SAFE.decode(base64.as_bytes()).unwrap();

    let result = blowfish.decrypt(&decoded).unwrap();

    let str = String::from_utf8(result).unwrap();
    str.trim_matches(char::from(0)).to_string()
}

pub fn encrypt_arguments(commandline: &str) -> String {
    let blowfish = initialize_blowfish();
    let encrypted = blowfish.encrypt(commandline.as_bytes()).unwrap();

    let encoded = URL_SAFE.encode(encrypted);
    let checksum = calculate_checksum(calculate_blowfish_key());

    format!("//**sqex0003{}{}**//", encoded, checksum)
}
