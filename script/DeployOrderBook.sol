// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Script.sol";
import "src/concrete/OrderBook.sol";

contract DeployOrderBook is Script {
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        // hardcoded from CI https://github.com/rainprotocol/rain-protocol/actions/runs/5365826502/jobs/9934721206
        address i9rDeployer = 0x12CC9A83C200354bc35e19e6ad18a0F444aB5c86;

        console2.log("meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        OrderBook deployed = new OrderBook(DeployerDiscoverableMetaV1ConstructionConfig (
            i9rDeployer,
            meta
        ));
        (deployed);
        vm.stopBroadcast();
    }
}
