// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOrderOwnerTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOrderBookHappy() external {
        checkSubParserContextHappy("order-owner");
    }
}
