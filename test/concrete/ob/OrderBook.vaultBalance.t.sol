// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";

/// @title OrderBookVaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookDepositTest is OrderBookExternalMockTest {
    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address token, uint256 vaultId) external view {
        assertEq(iOrderbook.vaultBalance(address(this), token, vaultId), 0);
    }
}
