// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextVaultBalanceBeforeTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextInputVaultBalanceBeforeHappy() external {
        checkSubParserContextHappy("input-vault-balance-before");
    }
}
