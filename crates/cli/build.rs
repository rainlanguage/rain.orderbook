use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=*");

    Command::new("nix")
        .args(["develop", "--command", "rainix-sol-prelude"]).output().unwrap();

    panic!("done");
}