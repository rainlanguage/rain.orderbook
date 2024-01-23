// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::vault::{vault_detail, vaults_list};
use commands::wallet::get_address_from_ledger;
use commands::fork::fork_call;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            vaults_list,
            vault_detail,
            get_address_from_ledger
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
