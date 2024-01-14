// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOutputTokenDecimalsTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOutputTokenDecimalsHappy() external {
        checkSubParserContextHappy("output-token-decimals");
    }
}
