// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script} from "forge-std/Script.sol";
import {OrderBook} from "src/concrete/OrderBook.sol";
import {I9R_DEPLOYER} from "./DeployConstants.sol";

contract DeployOrderBook is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        vm.startBroadcast(deployerPrivateKey);
        OrderBook deployed = new OrderBook(I9R_DEPLOYER);
        (deployed);
        vm.stopBroadcast();
    }
}
