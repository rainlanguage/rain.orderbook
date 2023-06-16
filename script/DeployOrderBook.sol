// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Script.sol";
import "src/OrderBook.sol";

contract DeployOrderBook is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        // @todo pull this from subgraph.
        // hardcoded from CI https://github.com/rainprotocol/rain-protocol/actions/runs/5039345251/jobs/9037426821
        address i9rDeployer = 0xB20DFEdC1b12AA6afA308064998A28531a18C714;

        string[] memory buildMeta = new string[](11);
        buildMeta[0] = "rain";
        buildMeta[1] = "meta";
        buildMeta[2] = "build";
        buildMeta[3] = "--magic";
        buildMeta[4] = "interpreter-caller-meta-v1";
        buildMeta[5] = "--input-path";
        buildMeta[6] = "meta/OrderBook.meta.json";
        buildMeta[7] = "--content-type";
        buildMeta[8] = "json";
        buildMeta[9] = "--content-encoding";
        buildMeta[10] = "deflate";

        bytes memory meta = vm.ffi(buildMeta);
        vm.startBroadcast(deployerPrivateKey);

        OrderBook orderbook = new OrderBook(DeployerDiscoverableMetaV1ConstructionConfig (
            i9rDeployer,
            meta
        ));
        (orderbook);

        vm.stopBroadcast();
    }
}