#![allow(static_mut_refs)]

mod config;
use config::get_config;

mod utilities;
use utilities::{find_signature, get_utf16_bytes, show_message};

use proxy_dll::proxy;
use skidscan::Signature;

/// So we can have a static address that won't change.
static mut OVERRIDE_URL: Vec<u8> = Vec::new();

/// The first string in the launcher URLs array.
const ABOUT_BLANK_URL: &str = "about:blank";

/// Filename of the launcher.
const LAUNCHER_FILENAME: &str = "ffxivlauncher64.exe";

fn overwrite_launcher_url() {
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
        OVERRIDE_URL.extend_from_slice(&get_utf16_bytes(&get_config().launcher_url));
        *launcher_url_launcher_page = OVERRIDE_URL.as_ptr() as u64;
    }
}

fn tweak_launcher() {
    overwrite_launcher_url();
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

    // We only care about the launcher for now
    if current_exe == LAUNCHER_FILENAME {
        tweak_launcher();
    }
}
