// use utils::{
//     deploy::{deploy1820::deploy1820, deploy_orderbook::deploy_orderbook},
//     utils::deploy_anvil_and_docker,
// };

// use ethers::contract::Abigen;

use std::process::Command;

mod utils;
#[tokio::main]
#[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    println!("Hello my man");
    // let command = "forge";
    // let args = vec!["build", "--root", "../"];

    // let mut cmd = Command::new(command);
    // cmd.args(args);

    // let output = cmd.output().expect("Failed to run command");

    // if output.status.success() {
    //     println!(
    //         "SUCCESS, OUTPUT: \n{}",
    //         String::from_utf8_lossy(&output.stdout)
    //     );
    // } else {
    //     eprintln!(
    //         "FAILED, OUTPUT: \n{}",
    //         String::from_utf8_lossy(&output.stdout)
    //     );
    // }

    assert!(true, "test_1");
    // Abigen::new("TokenReserve", "./ReserveToken.json")?
    //     .generate()?
    //     .write_to_file("token.rs")?;
    // let anvil = deploy_anvil_and_docker()?;
    // deploy1820(&anvil).await?;
    // deploy_orderbook(&anvil).await?;

    Ok(())
}
