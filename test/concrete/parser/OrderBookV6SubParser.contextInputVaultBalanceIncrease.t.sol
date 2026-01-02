// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6SubParserContextTest} from "test/util/abstract/OrderBookV6SubParserContextTest.sol";

contract OrderBookV6SubParserContextInputVaultBalanceIncreaseTest is OrderBookV6SubParserContextTest {
    function word() internal pure override returns (string memory) {
        return "input-vault-increase";
    }
}
