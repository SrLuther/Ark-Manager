// Ark Manager — Backend Rust (Tauri 2)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ark_manager_lib::run();
}
