// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script, console2} from "forge-std/Script.sol";
import {OrderBook, DeployerDiscoverableMetaV3ConstructionConfig} from "src/concrete/OrderBook.sol";

contract DeployOrderBook is Script {
    function run(bytes memory meta) external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        // hardcoded from CI https://github.com/rainprotocol/rain.interpreter/actions/runs/6953107467/job/18917750124
        address i9rDeployer = 0xa5aDC3F2A7A8Cf7b5172D76d8b26c3d49272297B;

        console2.log("DeployOrderBook meta hash:");
        console2.logBytes32(keccak256(meta));

        vm.startBroadcast(deployerPrivateKey);
        OrderBook deployed = new OrderBook(DeployerDiscoverableMetaV3ConstructionConfig(i9rDeployer, meta));
        (deployed);
        vm.stopBroadcast();
    }
}
