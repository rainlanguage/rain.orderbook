// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOrderCounterpartyTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOrderCounterpartyHappy() external {
        checkSubParserContextHappy("order-counterparty");
    }
}
