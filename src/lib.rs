#![allow(static_mut_refs)]

mod config;
use config::get_config;

mod utilities;
use utilities::{find_signature, find_symbol, get_utf16_bytes, show_message};

use proxy_dll::proxy;
use retour::static_detour;
use skidscan::Signature;
use std::ffi::CStr;
use std::ffi::c_char;
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

/// Domain of the retail game patch server.
const RETAIL_GAME_PATCH_SERVER: &str = "patch-gamever.ffxiv.com";

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
    static WinHttpConnect: fn(*mut c_void, PCWSTR, u16, u32) -> *mut c_void;
    static WinHttpOpenRequest: fn(*mut c_void, PCWSTR, PCWSTR, PCWSTR, PCWSTR, *mut c_void, u32) -> *mut c_void;
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

fn winhttpconnect_detour(
    hsession: *mut c_void,
    pswzservername: PCWSTR,
    nserverport: u16,
    dwreserved: u32,
) -> *mut c_void {
    let config = get_config();

    let server_name;
    unsafe {
        let original_server_name = pswzservername.to_string().unwrap();

        if original_server_name == RETAIL_GAME_PATCH_SERVER && config.game_patch_server.is_some() {
            server_name = config.game_patch_server.unwrap();
        } else {
            server_name = original_server_name;
        }
    }

    let http_proxy: Vec<u16> = server_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let port = if config.force_http { 80 } else { nserverport };

    // See https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    WinHttpConnect.call(hsession, PCWSTR(http_proxy.as_ptr() as _), port, dwreserved)
}

fn winhttpopenrequest_detour(
    hconnect: *mut c_void,
    pwszverb: PCWSTR,
    pwszobjectname: PCWSTR,
    pwszversion: PCWSTR,
    pwszreferrer: PCWSTR,
    ppwszaccepttypes: *mut c_void,
    _dwflags: u32,
) -> *mut c_void {
    // See https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    WinHttpOpenRequest.call(
        hconnect,
        pwszverb,
        pwszobjectname,
        pwszversion,
        pwszreferrer,
        ppwszaccepttypes,
        0, // kill secure connect flag
    )
}

fn overwrite_patch_url() {
    unsafe {
        let winhttpconnect_addr = find_symbol("WinHttpConnect", "winhttp.dll")
            .expect("Failed to find WinHttpConnect address");
        let winhttpconnect_fn = std::mem::transmute::<
            *mut u8,
            fn(*mut c_void, PCWSTR, u16, u32) -> *mut c_void,
        >(winhttpconnect_addr);
        WinHttpConnect
            .initialize(winhttpconnect_fn, winhttpconnect_detour)
            .expect("Failed to initialize WinHttpConnect hook");
        WinHttpConnect
            .enable()
            .expect("Failed to hook into WinHttpOpen");
    }
}

fn force_http() {
    let config = get_config();
    if !config.force_http {
        return;
    }

    unsafe {
        let winhttpopenrequest_addr = find_symbol("WinHttpOpenRequest", "winhttp.dll")
            .expect("Failed to find WinHttpOpenRequest address");
        let winhttpopenrequest_fn = std::mem::transmute::<
            *mut u8,
            fn(*mut c_void, PCWSTR, PCWSTR, PCWSTR, PCWSTR, *mut c_void, u32) -> *mut c_void,
        >(winhttpopenrequest_addr);
        WinHttpOpenRequest
            .initialize(winhttpopenrequest_fn, winhttpopenrequest_detour)
            .expect("Failed to initialize WinHttpOpenRequest hook");
        WinHttpOpenRequest
            .enable()
            .expect("Failed to hook into WinHttpOpenRequest");
    }
}

static_detour! {
    static InstallWebView2: fn(u64) -> i32;
}

fn install_webview2_detour(_unk: u64) -> i32 {
    // non-zero: will let us through, even if we don't have WebView2
    // 0 = will do the "webview2 failed to install message"
    return 1;
}

fn disable_webview2_install() {
    let config = get_config();
    if !config.disable_webview2_install {
        return;
    }

    unsafe {
        // TODO: wow, that func sig is baaaad
        let install_webview2_addr =
                        skidscan::signature!("48 89 5c 24 10 48 89 7c 24 18 55 48 8d 6c 24 c0 48 81 ec 40 01 00 00 48 8b 05 6a 4f 0f 00 48 33 c4 48 89 45 30 48 8b d9 e8 d3 cb ff ff").scan_module(BOOT_FILENAME).unwrap();
        let install_webview2_fn =
            std::mem::transmute::<*mut u8, fn(u64) -> i32>(install_webview2_addr);
        InstallWebView2
            .initialize(install_webview2_fn, install_webview2_detour)
            .expect("Failed to initialize InstallWebView2 hook");
        InstallWebView2
            .enable()
            .expect("Failed to hook into InstallWebView2");
    }
}

static_detour! {
    static GetConfigOption: fn(*const ConfigBase, u32) -> *mut ConfigEntry;
}

#[repr(C)]
struct ConfigEntry {
    _padding: [u8; 0x10],
    name: *const c_char,
    config_type: ConfigType,
    _padding2: [u8; 0x4],
    config_value: ConfigValue,
}

#[repr(i32)]
#[derive(Debug)]
enum ConfigType {
    Unused = 0,
    Category = 1,
    UInt = 2,
    Float = 3,
    String = 4,
}

#[repr(C)]
union ConfigValue {
    UInt: u32,
    Float: f32,
}

#[repr(C)]
struct ConfigBase {
    _padding: [u8; 0x18],
    ConfigEntry: *const ConfigEntry,
}

fn get_config_option_detour(
    config_base: *const ConfigBase,
    config_option: u32,
) -> *mut ConfigEntry {
    unsafe {
        let option = GetConfigOption.call(config_base, config_option);
        if option != std::ptr::null_mut() {
            if (*option).name != std::ptr::null() {
                let name = CStr::from_ptr((*option).name as *const i8);

                if name.to_str().unwrap() == "SkipBootupVercheck" {
                    (*option).config_value.UInt = 1;
                }
            }
        }

        option
    }
}

fn disable_boot_version_check() {
    let config = get_config();
    if !config.disable_boot_version_check {
        return;
    }

    unsafe {
        // TODO: *terrible* function man
        let get_config_option_addr = skidscan::signature!("48 89 5c 24 08 48 89 6c 24 10 48 89 74 24 18 57 48 83 ec 20 8b f2 48 8d b9 c8 00 00 00 48 8b e9 33 db").scan_module(BOOT_FILENAME).unwrap();
        let get_config_option_fn = std::mem::transmute::<
            *mut u8,
            fn(*const ConfigBase, u32) -> *mut ConfigEntry,
        >(get_config_option_addr);
        GetConfigOption
            .initialize(get_config_option_fn, get_config_option_detour)
            .expect("Failed to initialize GetConfigOption hook");
        GetConfigOption
            .enable()
            .expect("Failed to hook into GetConfigOption");
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
            overwrite_patch_url();
            force_http();
        }
        BOOT_FILENAME => {
            use_system_proxy();
            disable_webview2_install();
            disable_boot_version_check();
            force_http();
        }
        _ => {}
    }
}
