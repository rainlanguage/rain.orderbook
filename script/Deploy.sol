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

        vm.startBroadcast(deployerPrivateKey);

        // OB.
        OrderBook orderbook = new OrderBook();

        // Subparsers.
        new OrderBookSubParser();

        // Order takers.
        new GenericPoolOrderBookV3ArbOrderTaker();
        new RouteProcessorOrderBookV3ArbOrderTaker(OrderBookV3ArbOrderTakerConfigV1(
            address(orderbook),
            EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
        ));

        // Flash borrowers.
        new GenericPoolOrderBookV3FlashBorrower();

        vm.stopBroadcast();
    }
}
