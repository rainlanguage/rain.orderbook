// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod toast;
pub mod transaction_status;

mod commands;
use commands::chain::get_chainid;
use commands::fork::fork_call;
use commands::vault::{vault_deposit, vault_detail, vault_withdraw, vaults_list};
use commands::wallet::get_address_from_ledger;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            vaults_list,
            vault_detail,
            fork_call,
            vault_deposit,
            vault_withdraw,
            get_address_from_ledger,
            get_chainid
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
