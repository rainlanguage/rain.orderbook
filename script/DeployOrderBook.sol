// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script, console2} from "forge-std/Script.sol";
import {OrderBook, DeployerDiscoverableMetaV3ConstructionConfig} from "src/concrete/OrderBook.sol";
import {I9R_DEPLOYER} from "./DeployConstants.sol";

contract DeployOrderBook is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        bytes memory meta = hex"";

        vm.startBroadcast(deployerPrivateKey);
        OrderBook deployed = new OrderBook(DeployerDiscoverableMetaV3ConstructionConfig(I9R_DEPLOYER, meta));
        (deployed);
        vm.stopBroadcast();
    }
}
