use std::ffi::CString;

use skidscan::Signature;
use windows::{
    Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    Win32::UI::WindowsAndMessaging::{IDYES, MB_YESNO, MessageBoxA},
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

/// Asks whether you want to connect to a custom server, the official server.
pub fn ask_launcher_message() -> bool {
    unsafe {
        let message = CString::new("Do you want to connect to your custom server? Select \"No\" to connect to the official server.").unwrap();

        MessageBoxA(
            None,
            PCSTR::from_raw(message.into_raw() as *const u8),
            s!("LauncherTweaks"),
            MB_YESNO,
        ) == IDYES
    }
}

/// Writes to a temporary file that indicates whether or not the user chose to boot into the official server or not.
pub fn write_official_server_decision(launch_official_server: bool) {
    let mut server_file = std::env::temp_dir();
    server_file.push("launchertweaks_server.txt");

    let byte = if launch_official_server { 1u8 } else { 0u8 };
    std::fs::write(server_file, [byte]).unwrap();
}

/// Checks the decision file created during boot.
pub fn check_official_server_decision() -> bool {
    let mut server_file = std::env::temp_dir();
    server_file.push("launchertweaks_server.txt");

    let decision = std::fs::read(server_file).unwrap_or_default();
    decision == [1]
}

pub fn get_utf16_bytes(string: &str) -> Vec<u8> {
    unsafe {
        let utf16_bytes: Vec<u16> = string.encode_utf16().chain(std::iter::once(0)).collect();
        utf16_bytes.align_to::<u8>().1.to_vec()
    }
}
