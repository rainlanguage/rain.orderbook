pub mod cbor;
pub mod deploy;
pub mod events;
pub mod gen_abigen;
pub mod numbers;
pub mod setup;

pub use setup::get_provider;

use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::Middleware,
    signers::{coins_bip39::English, MnemonicBuilder, Wallet},
    types::{Bytes, U64},
};
use hex::FromHex;
use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};

pub async fn get_block_number() -> U64 {
    let provider = get_provider().await.expect("cannot get provider");
    return provider.get_block_number().await.unwrap();
}

/// Get the wallet test at the given index
pub fn get_wallet(index: u32) -> Wallet<SigningKey> {
    let mnemonic = std::fs::read_to_string("./test-mnemonic").expect("Test mnemonic not found");

    let wallet_builder = MnemonicBuilder::<English>::default().phrase(mnemonic.as_str());

    return wallet_builder
        .clone()
        .index(index)
        .expect(format!("MnemonicBuilder cannot get index {}", index).as_str())
        .build()
        .expect(format!("MnemonicBuilder cannot build wallet at the index {}", index).as_str());
}

// This function will work on the working directory
pub fn _run_cmd(main_cmd: &str, args: &[&str]) -> bool {
    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Create a new Command to run
    let mut cmd = Command::new(main_cmd);

    // Add the arguments
    cmd.args(args);

    // Set the directory from where the command wil run
    cmd.current_dir(&current_dir);

    // Tell what to do when try to print the process
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let full_cmd = format!("{} {}", main_cmd, args.join(" "));

    println!("Running: {}", full_cmd);

    // Execute the command
    let mut child = cmd
        .spawn()
        .expect(format!("Failed to run: {}", full_cmd).as_str());

    // Read and print stdout in a separate thread
    let stdout_child = child.stdout.take().expect("Failed to get stdout");
    let stdout_reader = BufReader::new(stdout_child);

    let stdout_handle = thread::spawn({
        move || {
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                }
            }
        }
    });

    // Read and print stderr in the main thread
    let stderr_reader = BufReader::new(child.stderr.take().expect("Failed to get stderr"));
    for line in stderr_reader.lines() {
        if let Ok(line) = line {
            eprintln!("{}", line);
        }
    }

    // Wait for the command to finish and get the exit status
    let status = child
        .wait()
        .expect(format!("Failed to wait: {}", full_cmd).as_str());

    // Wait for the stdout thread to finish
    stdout_handle.join().expect("Failed to join stdout thread");

    if status.success() {
        println!("Success: {}", full_cmd);
        return true;
    } else {
        eprintln!(
            "Fail: {} {}",
            full_cmd,
            format!("failed with exit code: {}", status.code().unwrap_or(-1))
        );

        return false;
    }
}

/// Rain Magic Numbers
pub struct MagicNumber;

impl MagicNumber {
    pub fn rain_meta_document_v1() -> Bytes {
        Bytes::from_hex("0xff0a89c674ee7874").unwrap()
    }

    pub fn solidity_abi_v2() -> Bytes {
        Bytes::from_hex("0xffe5ffb4a3ff2cde").unwrap()
    }

    pub fn op_meta_v1() -> Bytes {
        Bytes::from_hex("0xffe5282f43e495b4").unwrap()
    }

    pub fn interpreter_caller_meta_v1() -> Bytes {
        Bytes::from_hex("0xffc21bbf86cc199b").unwrap()
    }

    pub fn authoring_meta_v1() -> Bytes {
        Bytes::from_hex("0xffe9e3a02ca8e235").unwrap()
    }

    pub fn rainlang_v1() -> Bytes {
        Bytes::from_hex("0xff1c198cec3b48a7").unwrap()
    }

    pub fn dotrain_v1() -> Bytes {
        Bytes::from_hex("0xffdac2f2f37be894").unwrap()
    }

    pub fn expression_deployer_v2() -> Bytes {
        Bytes::from_hex("0xffdb988a8cd04d32").unwrap()
    }
}
