// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookExternalTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookVaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookDepositTest is OrderBookExternalTest {
    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address token, uint256 vaultId) external {
        assertEq(orderbook.vaultBalance(address(this), token, vaultId), 0);
    }

    /// Test that depositing can't reentrantly read the vault balance.
    function testVaultBalanceReentrant(
        address alice,
        uint256 vaultIdAlice,
        uint256 amount,
        address bob,
        address tokenBob,
        uint256 vaultIdBob
    ) external {
        vm.assume(amount > 0);
        vm.prank(alice);
        Reenteroor reenteroor = new Reenteroor();
        reenteroor.reenterWith(abi.encodeWithSelector(IOrderBookV3.vaultBalance.selector, bob, tokenBob, vaultIdBob));
        vm.expectRevert(ReentrancyGuardReentrantCall.selector);
        orderbook.deposit(address(reenteroor), vaultIdAlice, amount);
    }
}
