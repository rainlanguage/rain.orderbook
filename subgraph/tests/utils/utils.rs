use anyhow::anyhow;
use ethers::utils::{Anvil, AnvilInstance};
use std::process::Command;

pub fn deploy_anvil_and_docker() -> anyhow::Result<AnvilInstance> {
    let proiver = Anvil::new().port(8545u16).spawn();

    println!("Anvil deployed at : {}", proiver.endpoint());
    // let output = Command::new("bash")
    //     .args(&["-c", "docker-compose -f docker/docker-compose.yaml up -d"])
    //     .output()
    //     .unwrap();

    // if !output.status.success() {
    //     let stderr = format!("{}", String::from_utf8_lossy(&output.stderr.to_vec()));
    //     return Err(anyhow!(stderr));
    // }
    Ok(proiver)
}

pub fn stop_docker() -> anyhow::Result<()> {
    let output = Command::new("bash")
        .args(&[
            "-c",
            "docker-compose -f docker/docker-compose.yaml down && rm -rf docker/data ",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = format!("{}", String::from_utf8_lossy(&output.stderr.to_vec()));
        return Err(anyhow!(stderr));
    }
    Ok(())
}
