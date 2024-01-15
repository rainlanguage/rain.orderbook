// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOrderBookTest is OrderBookSubParserContextTest {
    function word() internal pure override returns (string memory) {
        return "orderbook";
    }
}
