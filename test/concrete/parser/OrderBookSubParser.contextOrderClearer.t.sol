// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOrderClearerTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOrderClearerHappy() external {
        checkSubParserContextHappy("order-clearer");
    }
}
