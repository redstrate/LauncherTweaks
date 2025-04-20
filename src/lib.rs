#![allow(static_mut_refs)]

mod config;
use config::get_config;

mod utilities;
use utilities::{find_signature, find_symbol, get_utf16_bytes, show_message};

use proxy_dll::proxy;
use retour::static_detour;
use skidscan::Signature;
use std::ffi::c_void;
use windows::{Win32::Networking::WinHttp::*, core::*};

/// So we can have a static address that won't change.
static mut OVERRIDE_URL: Vec<u8> = Vec::new();

/// The first string in the launcher URLs array.
const ABOUT_BLANK_URL: &str = "about:blank";

/// Filename of the launcher executable.
const LAUNCHER_FILENAME: &str = "ffxivlauncher64.exe";

/// Filename of the boot executable.
const BOOT_FILENAME: &str = "ffxivboot64.exe";

fn overwrite_launcher_url() {
    let config = get_config();
    let Some(launcher_url) = config.launcher_url else {
        return;
    };

    unsafe {
        // We are going to overwrite a portion of memory that contains the two main launcher URLs.
        //
        // - The first element is about:blank (which also happens to be customized to show the logo)
        // - The second element is the actual launcher URL (e.g. https://launcher.finalfantasyxiv.com/v720/)
        //
        // These are *not* the strings, this is an array of pointers to the actual string data.
        // The strings are also UTF-16 encoded, hence all of the conversion below.

        // We can re-use the existing signature scanning to find the about:blank string data:
        let about_url_address = find_signature(Signature::from(get_utf16_bytes(ABOUT_BLANK_URL)))
            .expect("Failed to find about:empty");

        // Then find the array by searching for this address:
        let url_array_signature =
            Signature::from((about_url_address as u64).to_le_bytes().to_vec());
        let url_array_address =
            find_signature(url_array_signature).expect("Failed to find launcher URL array") as u64;

        // Now we can access the second element of the array that we care about, the launcher URL!
        let launcher_url_launcher_page =
            (url_array_address + std::mem::size_of::<u64>() as u64) as *mut u64;

        // Override!!!!
        OVERRIDE_URL.extend_from_slice(&get_utf16_bytes(&launcher_url));
        *launcher_url_launcher_page = OVERRIDE_URL.as_ptr() as u64;
    }
}

static_detour! {
    static WinHttpOpen: fn(PCWSTR, WINHTTP_ACCESS_TYPE, PCWSTR, PCWSTR, u32) -> *mut c_void;
}

fn winhttpopen_detour(
    pszagentw: PCWSTR,
    _dwaccesstype: WINHTTP_ACCESS_TYPE,
    _pszproxyw: PCWSTR,
    pszproxybypassw: PCWSTR,
    dwflags: u32,
) -> *mut c_void {
    let config = get_config();
    let http_proxy = config
        .winhttp_proxy
        .expect("Failed to get winhttp_proxy config value");
    let http_proxy: Vec<u16> = http_proxy
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // See https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen
    WinHttpOpen.call(
        pszagentw,
        WINHTTP_ACCESS_TYPE_NAMED_PROXY,
        PCWSTR(http_proxy.as_ptr() as _),
        pszproxybypassw,
        dwflags,
    )
}

fn use_system_proxy() {
    let config = get_config();
    if config.winhttp_proxy.is_none() {
        return;
    }

    unsafe {
        let winhttpopen_addr =
            find_symbol("WinHttpOpen", "winhttp.dll").expect("Failed to find WinHttpOpen address");
        let winhttpopen_fn = std::mem::transmute::<
            *mut u8,
            fn(PCWSTR, WINHTTP_ACCESS_TYPE, PCWSTR, PCWSTR, u32) -> *mut c_void,
        >(winhttpopen_addr);
        WinHttpOpen
            .initialize(winhttpopen_fn, winhttpopen_detour)
            .expect("Failed to initialize WinHttpOpen hook");
        WinHttpOpen
            .enable()
            .expect("Failed to hook into WinHttpOpen");
    }
}

#[proxy]
fn main() {
    // Emit panic messages with message boxes
    std::panic::set_hook(Box::new(|panic_hook_info| {
        let s = if let Some(s) = panic_hook_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_hook_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown error".to_string()
        };

        show_message(&format!("Error: {s}"));
    }));

    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let current_exe = current_exe
        .file_name()
        .expect("Failed to get executable filename");

    match current_exe
        .to_str()
        .expect("Failed to parse executable filename")
    {
        LAUNCHER_FILENAME => {
            use_system_proxy();
            overwrite_launcher_url();
        }
        BOOT_FILENAME => {
            use_system_proxy();
        }
        _ => {}
    }
}
