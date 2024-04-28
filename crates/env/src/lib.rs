pub const CI_DEPLOY_POLYGON_RPC_URL: &str = env!(
    "CI_DEPLOY_POLYGON_RPC_URL",
    "$CI_DEPLOY_POLYGON_RPC_URL not set."
);

pub const CI_DEPLOY_SEPOLIA_RPC_URL: &str = env!(
    "CI_DEPLOY_SEPOLIA_RPC_URL",
    "$CI_DEPLOY_SEPOLIA_RPC_URL not set."
);

pub const CI_SEPOLIA_METABOARD_URL: &str = env!(
    "CI_SEPOLIA_METABOARD_URL",
    "$CI_SEPOLIA_METABOARD_URL not set."
);

pub const CI_RPC_URL_ETHEREUM_FORK: &str =
    env!("RPC_URL_ETHEREUM_FORK", "$RPC_URL_ETHEREUM_FORK not set.");
