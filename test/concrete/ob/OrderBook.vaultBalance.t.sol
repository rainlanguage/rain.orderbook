// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/forge-std/src/Test.sol";

import "test/util/abstract/OrderBookExternalMockTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookVaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookDepositTest is OrderBookExternalMockTest {
    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address token, uint256 vaultId) external {
        assertEq(iOrderbook.vaultBalance(address(this), token, vaultId), 0);
    }
}
