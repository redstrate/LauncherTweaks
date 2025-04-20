use std::ffi::CString;

use skidscan::Signature;
use windows::{Win32::UI::WindowsAndMessaging::MessageBoxA, core::*};

use crate::LAUNCHER_FILENAME;

pub fn find_signature(sig: Signature) -> Option<*mut u8> {
    unsafe { sig.scan_module(LAUNCHER_FILENAME).ok() }
}

pub fn show_message(message: &str) {
    unsafe {
        let message = CString::new(message).unwrap();

        MessageBoxA(
            None,
            PCSTR::from_raw(message.into_raw() as *const u8),
            s!("LauncherTweaks"),
            Default::default(),
        );
    }
}

pub fn get_utf16_bytes(string: &str) -> Vec<u8> {
    unsafe {
        let utf16_bytes: Vec<u16> = string.encode_utf16().collect();
        utf16_bytes.align_to::<u8>().1.to_vec()
    }
}
