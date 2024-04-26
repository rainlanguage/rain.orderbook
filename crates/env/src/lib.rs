use alloy_primitives::{Address, BlockNumber};
use once_cell::sync::Lazy;

pub const CI_DEPLOY_POLYGON_RPC_URL: &str = env!(
    "CI_DEPLOY_POLYGON_RPC_URL",
    "$CI_DEPLOY_POLYGON_RPC_URL not set."
);
pub const CI_DEPLOY_SEPOLIA_RPC_URL: &str = env!(
    "CI_DEPLOY_SEPOLIA_RPC_URL",
    "$CI_DEPLOY_SEPOLIA_RPC_URL not set."
);
pub static CI_METABOARD_URL: Lazy<Address> = Lazy::new(|| {
    env!("CI_METABOARD_URL", "$CI_METABOARD_URL not set.")
        .parse()
        .unwrap()
});
