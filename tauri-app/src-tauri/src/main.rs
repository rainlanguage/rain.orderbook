// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::vault::{vaults_list, vault_detail};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![vaults_list, vault_detail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
