// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|argument| argument == "--plugin-host") {
        if let Err(error) = arclume_lib::run_plugin_host() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    arclume_lib::run()
}
