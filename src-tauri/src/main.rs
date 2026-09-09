// Keeps the console window from appearing alongside the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--version")) {
        println!("TrustVault {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    trustvault_lib::run()
}
