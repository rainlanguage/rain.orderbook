// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script} from "forge-std/Script.sol";
import {GenericPoolOrderBookV3ArbOrderTaker} from "src/concrete/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {I9R_DEPLOYER} from "./DeployConstants.sol";

/// @title DeployGenericPoolOrderBookV3ArbOrderTaker
/// @notice A script that deploys a `GenericPoolOrderBookV3ArbOrderTaker`. This
/// is intended to be run on every commit by CI to a testnet such as mumbai, then
/// cross chain deployed to whatever mainnet is required, by users.
contract DeployGenericPoolOrderBookV3ArbOrderTaker is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        vm.startBroadcast(deployerPrivateKey);
        GenericPoolOrderBookV3ArbOrderTaker deployed = new GenericPoolOrderBookV3ArbOrderTaker(I9R_DEPLOYER);
        (deployed);
        vm.stopBroadcast();
    }
}
