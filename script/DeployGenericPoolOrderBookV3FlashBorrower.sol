// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script} from "forge-std/Script.sol";
import {GenericPoolOrderBookV3FlashBorrower} from "src/concrete/GenericPoolOrderBookV3FlashBorrower.sol";
import {I9R_DEPLOYER} from "./DeployConstants.sol";

/// @title DeployGenericPoolOrderBookV3FlashBorrower
/// @notice A script that deploys a `GenericPoolOrderBookV3FlashBorrower`. This
/// is intended to be run on every commit by CI to a testnet such as mumbai, then
/// cross chain deployed to whatever mainnet is required, by users.
contract DeployGenericPoolOrderBookV3FlashBorrower is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        vm.startBroadcast(deployerPrivateKey);
        GenericPoolOrderBookV3FlashBorrower deployed = new GenericPoolOrderBookV3FlashBorrower(I9R_DEPLOYER);
        (deployed);
        vm.stopBroadcast();
    }
}
