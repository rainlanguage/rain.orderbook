// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextOutputVaultBalanceDecreaseTest is OrderBookSubParserContextTest {
    function word() internal pure override returns (string memory) {
        return "output-vault-decrease";
    }
}
