// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookSubParserContextTest} from "test/util/abstract/OrderBookSubParserContextTest.sol";

contract OrderBookSubParserContextInputVaultBalanceIncreaseTest is OrderBookSubParserContextTest {
    function word() internal pure override returns (string memory) {
        return "input-vault-increase";
    }
}
