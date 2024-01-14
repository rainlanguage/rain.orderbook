// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextInputTokenDecimalsTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextInputTokenDecimalsHappy() external {
        checkSubParserContextHappy("input-token-decimals");
    }
}
