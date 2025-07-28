// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title OrderBookVaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookVaultBalanceTest is OrderBookExternalMockTest {
    using LibDecimalFloat for Float;
    /// Test that reading the vault balance without deposits is always zero.

    function testVaultBalanceNoDeposits(address token, bytes32 vaultId) external view {
        assertTrue(iOrderbook.vaultBalance2(address(this), token, vaultId).isZero());
    }
}
