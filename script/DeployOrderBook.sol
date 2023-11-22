// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script, console2} from "forge-std/Script.sol";
import {OrderBook, DeployerDiscoverableMetaV3ConstructionConfig} from "src/concrete/OrderBook.sol";

contract DeployOrderBook is Script {
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        // hardcoded from CI https://github.com/rainprotocol/rain.interpreter/actions/runs/6101787278/job/16558857505
        address i9rDeployer = 0xAb0A13cC2654CbaDABabC9952a090928F4ff569A;

        console2.log("DeployOrderBook meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        OrderBook deployed = new OrderBook(DeployerDiscoverableMetaV3ConstructionConfig (
            i9rDeployer,
            meta
        ));
        (deployed);
        vm.stopBroadcast();
    }
}
