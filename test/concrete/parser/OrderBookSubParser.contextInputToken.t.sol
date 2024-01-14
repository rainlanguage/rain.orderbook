// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextInputTokenTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextInputTokenHappy() external {
        checkSubParserContextHappy("input-token");
    }
}
