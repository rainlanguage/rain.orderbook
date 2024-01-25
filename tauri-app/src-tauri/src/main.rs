// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::fork::fork_call;
use commands::vault::{vault_detail, vaults_list};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            vaults_list,
            vault_detail,
            fork_call
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
