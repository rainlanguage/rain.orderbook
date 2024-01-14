// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOutputTokenTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOutputTokenHappy() external {
        checkSubParserContextHappy("output-token");
    }
}
