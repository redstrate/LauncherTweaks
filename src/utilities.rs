use std::ffi::CString;

use skidscan::Signature;
use windows::{
    Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    Win32::UI::WindowsAndMessaging::MessageBoxA,
    core::*,
};

use crate::LAUNCHER_FILENAME;

pub fn find_signature(sig: Signature) -> Option<*mut u8> {
    unsafe { sig.scan_module(LAUNCHER_FILENAME).ok() }
}

pub fn find_symbol(symbol: &str, module: &str) -> Option<*mut u8> {
    let module: Vec<u16> = module.encode_utf16().chain(std::iter::once(0)).collect();
    let symbol = CString::new(symbol).expect("Failed to parse symbol!");
    unsafe {
        let handle = GetModuleHandleW(PCWSTR(module.as_ptr() as _)).expect("Module not loaded!");
        GetProcAddress(handle, PCSTR(symbol.as_ptr() as _)).map(|func| func as *mut u8)
    }
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
