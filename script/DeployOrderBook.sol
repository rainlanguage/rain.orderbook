// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script, console2} from "forge-std/Script.sol";
import {OrderBook, DeployerDiscoverableMetaV3ConstructionConfig} from "src/concrete/OrderBook.sol";
import {I9R_DEPLOYER} from "./DeployConstants.sol";

contract DeployOrderBook is Script {
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        console2.log("DeployOrderBook meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        OrderBook deployed = new OrderBook(DeployerDiscoverableMetaV3ConstructionConfig(I9R_DEPLOYER, meta));
        (deployed);
        vm.stopBroadcast();
    }
}
