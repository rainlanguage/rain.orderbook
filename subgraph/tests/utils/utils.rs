use web3::{transports::Http, Web3};

pub fn get_web3() -> Web3<Http> {
    let web3 = web3::Web3::new(Http::new("http://localhost:8545").unwrap());
    web3
}
