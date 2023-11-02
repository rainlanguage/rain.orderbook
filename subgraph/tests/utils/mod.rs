pub mod cbor;
pub mod deploy;
pub mod events;
pub mod gen_abigen;
pub mod json_structs;
pub mod numbers;
pub mod setup;
pub mod transactions;

use ethers::{
    core::{k256::ecdsa::SigningKey, rand::random, utils::hex},
    providers::Middleware,
    signers::{coins_bip39::English, MnemonicBuilder, Wallet},
    types::{Address, Bytes, H256, U256, U64},
};
use hex::FromHex;
use rust_bigint::BigInt;
pub use setup::get_provider;
use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};
use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

pub async fn get_block_number() -> anyhow::Result<U64> {
    let provider = get_provider().await?;
    Ok(provider.get_block_number().await?)
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

/// Convert an string to bytes using their ASCII corresponding values.
pub fn ascii_string_to_bytes(value: String) -> Bytes {
    Bytes::from(value.as_bytes().to_vec())
}

/// Convert an hexadecimal string to bytes
pub fn hex_string_to_bytes(value: &str) -> anyhow::Result<Bytes> {
    Ok(Bytes::from_hex(value)?)
}

pub fn _remove_trailing_zeros(arr: &[u8]) -> Vec<u8> {
    // Find the position of the last non-zero element
    let length = arr.iter().rposition(|&x| x != 0).map(|pos| pos + 1);

    match length {
        Some(len) => {
            // Create a new Vec<u8> with the non-zero data
            arr[0..len].to_vec()
        }
        // All elements are zeros, so return just one zero
        None => vec![0],
    }
}

/// Parse a given `BigInt`/`Mpz(rust_bigint::BigInt)` comming from Subgraph responses
/// to a value to an `ethers::U256`.
/// ### NOTE:
/// For some reason, the BigInt comming from Subgraph responses consider the BigInt
/// as an hexadecimal value and parse it (internally) to a decimal. In this function
/// the parse logic is made considering that, so parse it from "hex" to "decimal".
pub fn mn_mpz_to_u256(value: &BigInt) -> U256 {
    U256::from_dec_str(&value.to_str_radix(16)).unwrap()
}

/// Take a Bytes value and parse it to an H256. Take in count that if the Bytes value
/// is bigger than 32 Bytes, will be truncated.
pub fn bytes_to_h256(value: &Bytes) -> H256 {
    H256::from_slice(value.to_vec().as_slice())
}

/// Take a H256 value and parse it to a Bytes
pub fn h256_to_bytes(value: &H256) -> Bytes {
    Bytes::from(value.as_bytes().to_vec())
}

/// Take a U256 value and parse it to a Bytes
pub fn u256_to_bytes(value: &U256) -> Bytes {
    let mut bb: [u8; 32] = [0; 32];
    value.to_big_endian(&mut bb);

    ethers::types::Bytes::from(bb)
}

/// Take a U256 value and parse it to an Address
pub fn u256_to_address(value: &U256) -> Address {
    let bytes = u256_to_bytes(value);

    Address::from_slice(&bytes[12..])
}

/// Get a mock encoded rain document with hardcoded data.
/// Does not contain any well info. Only rain doc well formed.
pub fn mock_rain_doc() -> Bytes {
    Bytes::from_hex("0xff0a89c674ee7874a30052746869735f69735f616e5f6578616d706c65011bffe5ffb4a3ff2cde02706170706c69636174696f6e2f6a736f6e").unwrap()
}

pub fn generate_random_u256() -> U256 {
    // This trully is a random u64, but it's work for testing
    return U256::from(random::<u64>());
}

pub fn hash_keccak(data: &Vec<u8>) -> H256 {
    let mut keccak = Keccak::v256();
    keccak.update(&data);

    let mut output = [0u8; 32];
    keccak.finalize(&mut output);

    return H256::from(output);
}

/// Rain Magic Numbers
pub struct MagicNumber;

impl MagicNumber {
    pub fn rain_meta_document_v1() -> Bytes {
        Bytes::from_hex("0xff0a89c674ee7874").unwrap()
    }

    pub fn _solidity_abi_v2() -> Bytes {
        Bytes::from_hex("0xffe5ffb4a3ff2cde").unwrap()
    }

    pub fn _op_meta_v1() -> Bytes {
        Bytes::from_hex("0xffe5282f43e495b4").unwrap()
    }

    pub fn _interpreter_caller_meta_v1() -> Bytes {
        Bytes::from_hex("0xffc21bbf86cc199b").unwrap()
    }

    pub fn _authoring_meta_v1() -> Bytes {
        Bytes::from_hex("0xffe9e3a02ca8e235").unwrap()
    }

    pub fn _rainlang_v1() -> Bytes {
        Bytes::from_hex("0xff1c198cec3b48a7").unwrap()
    }

    pub fn _dotrain_v1() -> Bytes {
        Bytes::from_hex("0xffdac2f2f37be894").unwrap()
    }

    pub fn _expression_deployer_v2() -> Bytes {
        Bytes::from_hex("0xffdb988a8cd04d32").unwrap()
    }
}
