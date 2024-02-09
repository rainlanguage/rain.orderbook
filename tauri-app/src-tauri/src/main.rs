// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod error;
pub mod toast;
pub mod transaction_status;

mod commands;
use commands::chain::get_chainid;
use commands::fork::parse_dotrain;
use commands::lsp_services::{provide_completion, provide_hover, provide_problems};
use commands::order::{order_add, order_detail, order_remove, orders_list, orders_list_write_csv};
use commands::vault::{
    vault_deposit, vault_detail, vault_list_balance_changes, vault_list_balance_changes_write_csv,
    vault_withdraw, vaults_list, vaults_list_write_csv,
};
use commands::order_clear::{order_clears_list, order_clears_list_write_csv};
use commands::wallet::get_address_from_ledger;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            vaults_list,
            vaults_list_write_csv,
            vault_list_balance_changes,
            vault_list_balance_changes_write_csv,
            vault_detail,
            vault_deposit,
            vault_withdraw,
            orders_list,
            orders_list_write_csv,
            order_detail,
            order_add,
            order_remove,
            order_clears_list,
            order_clears_list_write_csv,
            get_address_from_ledger,
            get_chainid,
            parse_dotrain,
            provide_completion,
            provide_hover,
            provide_problems
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
