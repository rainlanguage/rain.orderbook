// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IOrderBookV6} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";

/// @title OrderBookV6VaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookV6VaultBalanceTest is OrderBookV6ExternalMockTest {
    using LibDecimalFloat for Float;

    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address owner, address token, bytes32 vaultId) external view {
        vm.assume(vaultId != bytes32(0));
        assertTrue(iOrderbook.vaultBalance2(owner, token, vaultId).isZero());
    }

    /// Vault balance for VaultId 0 is always an error.
    function testVaultBalanceZeroVaultIdReverts(address sender, address owner, address token) external {
        vm.prank(sender);
        vm.expectRevert(abi.encodeWithSelector(IOrderBookV6.ZeroVaultId.selector, owner, token));
        iOrderbook.vaultBalance2(owner, token, bytes32(0));
    }
}
