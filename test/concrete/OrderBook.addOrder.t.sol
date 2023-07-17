// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "test/util/abstract/OrderBookExternalRealTest.sol";

/// @title OrderBookAddOrderTest
/// @notice A test harness for testing the OrderBook addOrder function.
contract OrderBookAddOrderTest is OrderBookExternalRealTest {

    /// No sources reverts as we need at least a calculate expression.
    function testAddOrderRealNoSourcesReverts(OrderConfig memory config) public {
        // iOrderbook.addOrder(config);
    }
}