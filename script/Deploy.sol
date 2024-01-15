// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script} from "forge-std/Script.sol";
import {OrderBook} from "src/concrete/ob/OrderBook.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {GenericPoolOrderBookV3ArbOrderTaker} from "src/concrete/arb/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {RouteProcessorOrderBookV3ArbOrderTaker} from "src/concrete/arb/RouteProcessorOrderBookV3ArbOrderTaker.sol";
import {GenericPoolOrderBookV3FlashBorrower} from "src/concrete/arb/GenericPoolOrderBookV3FlashBorrower.sol";

/// @title Deploy
/// @notice A script that deploys all contracts. This is intended to be run on
/// every commit by CI to a testnet such as mumbai.
contract Deploy is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");

        address defaultExpressionDeployer = address(0);
        string memory deploymentPath = "lib/rain.interpreter/deployments/latest/RainterpreterExpressionDeployerNPE2";
        if (vm.isFile(deploymentPath)) {
            string memory fileContents = vm.readFile(deploymentPath);
            defaultExpressionDeployer = vm.parseAddress(fileContents);
        }
        address expressionDeployer = vm.envOr("EXPRESSION_DEPLOYER", defaultExpressionDeployer);

        vm.startBroadcast(deployerPrivateKey);

        // OB.
        new OrderBook(expressionDeployer);

        // Subparsers.
        new OrderBookSubParser();

        // Order takers.
        new GenericPoolOrderBookV3ArbOrderTaker(expressionDeployer);
        new RouteProcessorOrderBookV3ArbOrderTaker(expressionDeployer);

        // Flash borrowers.
        new GenericPoolOrderBookV3FlashBorrower(expressionDeployer);

        vm.stopBroadcast();
    }
}
