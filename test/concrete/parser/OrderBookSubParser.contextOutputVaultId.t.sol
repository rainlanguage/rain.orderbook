// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOrderBookTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOutputTokenVaultIDHappy() external {
        checkSubParserContextHappy("output-vault-id");
    }
}
