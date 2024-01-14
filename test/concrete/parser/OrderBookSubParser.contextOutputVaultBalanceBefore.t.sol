// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOutputVaultBalanceBeforeTest is OrderBookSubParserContextTest {
    function testOrderBookSubParserContextOutputVaultBalanceBeforeHappy() external {
        checkSubParserContextHappy("output-vault-balance-before");
    }
}
