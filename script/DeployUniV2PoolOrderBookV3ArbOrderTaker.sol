// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Script.sol";
import "src/concrete/UniV2PoolOrderBookV3ArbOrderTaker.sol";

/// @title DeployUniV2PoolOrderBookV3ArbOrderTaker
/// @notice A script that deploys a `UniV2PoolOrderBookV3ArbOrderTaker`. This
/// is intended to be run on every commit by CI to a testnet such as mumbai, then
/// cross chain deployed to whatever mainnet is required, by users.
contract DeployUniV2PoolOrderBookV3ArbOrderTaker is Script {
    /// We are avoiding using ffi here, instead forcing the script runner to
    /// provide the built metadata. On CI this is achieved by using the rain cli.
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        // hardcoded from CI https://github.com/rainprotocol/rain.interpreter/actions/runs/6062778321/job/16449363038
        address i9rDeployer = 0xCDAd930d648C562B570490901EC6d01D368B3b63;

        console2.log("UniV2PoolOrderBookV3ArbOrderTaker meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        GenericPoolOrderBookV3FlashBorrower deployed =
        new UniV2PoolOrderBookV3ArbOrderTaker(DeployerDiscoverableMetaV2ConstructionConfig (
            i9rDeployer,
            meta
        ));
        (deployed);
        vm.stopBroadcast();
    }
}
