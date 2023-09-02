// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Script.sol";
import "src/concrete/UniV2PoolOrderBookV3ArbOrderTaker.sol";

/// @title DeployGenericPoolOrderBookV3FlashBorrower
/// @notice A script that deploys a `GenericPoolOrderBookV3FlashBorrower`.
contract DeployUniV2PoolOrderBookV3ArbOrderTaker is Script {
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        //https://github.com/rainprotocol/rain.interpreter/actions/runs/6000630847/job/16273099850
        address i9rDeployer = 0xCA0Ef6E0d9cd47d44aF5d87098f8482669303b06;

        console2.log("DeployUniV2PoolOrderBookV3ArbOrderTaker meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        UniV2PoolOrderBookV3ArbOrderTaker deployed =
        new UniV2PoolOrderBookV3ArbOrderTaker(DeployerDiscoverableMetaV2ConstructionConfig (
            i9rDeployer,
            meta
        ));
        (deployed);
        vm.stopBroadcast();
    }
}
