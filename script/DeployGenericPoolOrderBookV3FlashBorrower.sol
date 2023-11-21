// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/forge-std/src/Script.sol";
import "src/concrete/GenericPoolOrderBookV3FlashBorrower.sol";

/// @title DeployGenericPoolOrderBookV3FlashBorrower
/// @notice A script that deploys a `GenericPoolOrderBookV3FlashBorrower`. This
/// is intended to be run on every commit by CI to a testnet such as mumbai, then
/// cross chain deployed to whatever mainnet is required, by users.
contract DeployGenericPoolOrderBookV3FlashBorrower is Script {
    /// We are avoiding using ffi here, instead forcing the script runner to
    /// provide the built metadata. On CI this is achieved by using the rain cli.
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        // hardcoded from CI https://github.com/rainprotocol/rain.interpreter/actions/runs/6101787278/job/16558857505
        address i9rDeployer = 0xAb0A13cC2654CbaDABabC9952a090928F4ff569A;

        console2.log("DeployGenericPoolOrderBookV3FlashBorrower meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        GenericPoolOrderBookV3FlashBorrower deployed =
        new GenericPoolOrderBookV3FlashBorrower(DeployerDiscoverableMetaV3ConstructionConfig (
            i9rDeployer,
            meta
        ));
        (deployed);
        vm.stopBroadcast();
    }
}
