// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOrderHashTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOrderBookHappy() external {
        checkSubParserContextHappy("order-hash");
    }
}
