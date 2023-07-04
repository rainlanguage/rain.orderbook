// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookTest.sol";

/// @title OrderBookVaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookDepositTest is OrderBookTest {
    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address token, uint256 vaultId) external {
        assertEq(orderbook.vaultBalance(address(this), token, vaultId), 0);
    }
}