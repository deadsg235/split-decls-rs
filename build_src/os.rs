// build_src/os.rs
// This module conditionally includes OS-specific build logic.

#[cfg(target_os = "linux")]
#[path = "os/linux.rs"]
mod os_impl;

#[cfg(target_os = "windows")]
#[path = "os/windows.rs"]
mod os_impl;

// If no specific OS is matched, you might want a fallback or an error
// #[cfg(not(any(target_os = "linux", target_os = "windows")))]
// #[path = "os/unsupported.rs"] // Create this file if you need a fallback
// mod os_impl;


pub fn run() {
    println!("cargo:warning=Running OS-specific build logic wrapper.");
    os_impl::run();
}
