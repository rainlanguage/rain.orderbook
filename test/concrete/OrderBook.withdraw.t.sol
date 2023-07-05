// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookWithdrawTest
/// Tests withdrawing from the order book.
contract OrderBookWithdrawTest is OrderBookTest {
    /// Withdrawing a zero target amount should revert.
    function testWithdrawZero(address alice, address token, uint256 vaultId) external {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(ZeroWithdrawTargetAmount.selector, alice, token, vaultId));
        orderbook.withdraw(token, vaultId, 0);
    }

    /// Withdrawing a non-zero amount from an empty vault should be a noop.
    function testWithdrawEmptyVault(address alice, address token, uint256 vaultId, uint256 amount) external {
        vm.assume(amount > 0);
        vm.prank(alice);
        vm.record();
        vm.recordLogs();
        orderbook.withdraw(token, vaultId, amount);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(orderbook));
        (reads);
        // Two writes to cover the reentrancy guard.
        assertEq(writes.length, 2);
        // Zero logs because nothing happened.
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 0);
    }



    /// Test that withdrawing can't reentrantly read the vault balance.
    function testWithdrawReentrant(
        address alice,
        uint256 vaultIdAlice,
        uint256 amount,
        address bob,
        address tokenBob,
        uint256 vaultIdBob
    ) external {
        vm.assume(amount > 0);
        Reenteroor reenteroor = new Reenteroor();
        reenteroor.reenterWith(abi.encodeWithSelector(IOrderBookV3.vaultBalance.selector, bob, tokenBob, vaultIdBob));

        // Withdraw short circuits if there's no vault balance so first we need
        // to deposit.
        vm.prank(alice);
        vm.mockCall(
            address(reenteroor),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(orderbook), amount),
            abi.encode(true)
        );
        orderbook.deposit(address(reenteroor), vaultIdAlice, amount);

        vm.prank(alice);
        vm.expectRevert(ReentrancyGuardReentrantCall.selector);
        orderbook.withdraw(address(reenteroor), vaultIdAlice, amount);
    }
}
