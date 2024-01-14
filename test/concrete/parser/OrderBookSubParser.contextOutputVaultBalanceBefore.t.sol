// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOutputVaultBalanceBeforeTest is OrderBookSubParserContextTest {
    function word() internal pure override returns (string memory) {
        return "output-vault-balance-before";
    }
}
