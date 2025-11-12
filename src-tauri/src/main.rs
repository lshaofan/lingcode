// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize tracing/logging system
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false) // Don't show module path in logs
        .with_thread_ids(false)
        .with_line_number(false)
        .init();

    println!("🚀 Starting Lingcode application...");

    lingcode_lib::run()
}
