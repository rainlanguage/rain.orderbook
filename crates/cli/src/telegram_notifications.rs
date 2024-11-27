use std::env;
use std::path::Path;
use std::process::Command;

pub fn send_telegram_notification() {
    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {:?}", current_dir);

    // Construct the absolute path to the script (going one level up from current_dir)
    let script_path = current_dir
        .parent()
        .expect("Failed to get parent directory")
        .join("send_telegram_message.sh");
    println!("Script path: {:?}", script_path);

    let output = Command::new(script_path)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        println!("Error sending telegram message: {:?}", output);
    }
}

pub fn check_balance(balance: u64) {
    if balance < 10 {
        send_telegram_notification();
    }
}
