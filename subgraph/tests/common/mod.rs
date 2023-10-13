pub mod deploy;
pub mod query;
pub mod wait;

use web3::{transports::Http, Web3};

pub fn get_web3() -> anyhow::Result<Web3<Http>> {
    let web3 = web3::Web3::new(Http::new("http://localhost:8545")?);
    Ok(web3)
}
