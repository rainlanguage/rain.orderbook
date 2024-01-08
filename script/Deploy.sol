// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script} from "forge-std/Script.sol";

// hardcoded from CI https://github.com/rainprotocol/rain.interpreter/actions/runs/6953107467/job/18917750124
address constant I9R_DEPLOYER = 0xa5aDC3F2A7A8Cf7b5172D76d8b26c3d49272297B;

/// @title Deploy
/// @notice A script that deploys all contracts. This is intended to be run on
/// every commit by CI to a testnet such as mumbai.
contract Deploy is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        vm.startBroadcast(deployerPrivateKey);

        // OB.
        new OrderBook(I9R_DEPLOYER);


        // Order takers.
        new GenericPoolOrderBookV3ArbOrderTaker(I9R_DEPLOYER);
        new RouteProcessorOrderBookV3ArbOrderTaker(I9R_DEPLOYER);

        // Flash borrowers.
        new GenericPoolOrderBookV3FlashBorrower(I9R_DEPLOYER);

        vm.stopBroadcast();
    }
}