// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";

import "test/util/abstract/OrderBookTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookWithdrawTest
/// Tests withdrawing from the order book.
contract OrderBookWithdrawTest is OrderBookTest {
    using Math for uint256;

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

    /// Withdrawing the full amount from a vault should delete the vault.
    function testWithdrawFullVault(address alice, uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount)
        external
    {
        vm.assume(depositAmount > 0);
        vm.assume(withdrawAmount >= depositAmount);
        vm.prank(alice);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(orderbook), depositAmount),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, depositAmount);
        assertEq(orderbook.vaultBalance(address(alice), address(token0), vaultId), depositAmount);

        vm.prank(alice);
        vm.mockCall(
            address(token0), abi.encodeWithSelector(IERC20.transfer.selector, alice, depositAmount), abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit Withdraw(alice, address(token0), vaultId, withdrawAmount, depositAmount);
        orderbook.withdraw(address(token0), vaultId, withdrawAmount);
        assertEq(orderbook.vaultBalance(address(alice), address(token0), vaultId), 0);
    }

    /// Withdrawing a partial amount from a vault should reduce the vault balance.
    function testWithdrawPartialVault(address alice, uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount)
        external
    {
        vm.assume(depositAmount > 0);
        vm.assume(withdrawAmount > 0);
        vm.assume(withdrawAmount < depositAmount);
        vm.prank(alice);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(orderbook), depositAmount),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, depositAmount);
        assertEq(orderbook.vaultBalance(address(alice), address(token0), vaultId), depositAmount);

        vm.prank(alice);
        vm.mockCall(
            address(token0), abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount), abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        // The full withdraw amount is possible as it's only a partial withdraw.
        emit Withdraw(alice, address(token0), vaultId, withdrawAmount, withdrawAmount);
        orderbook.withdraw(address(token0), vaultId, withdrawAmount);
        // The vault balance is reduced by the withdraw amount.
        assertEq(orderbook.vaultBalance(address(alice), address(token0), vaultId), depositAmount - withdrawAmount);
    }

    /// Any failure in the withdrawal should revert the entire transaction.
    function testWithdrawFailure(address alice, uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount)
        external
    {
        vm.assume(depositAmount > 0);
        vm.assume(withdrawAmount > 0);
        vm.prank(alice);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(orderbook), depositAmount),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, depositAmount);
        assertEq(orderbook.vaultBalance(address(alice), address(token0), vaultId), depositAmount);

        // The token contract always reverts when not mocked.
        vm.prank(alice);
        vm.expectRevert("SafeERC20: low-level call failed");
        orderbook.withdraw(address(token0), vaultId, withdrawAmount);

        vm.prank(alice);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount.min(depositAmount)),
            abi.encode(false)
        );
        vm.expectRevert("SafeERC20: ERC20 operation did not succeed");
        orderbook.withdraw(address(token0), vaultId, withdrawAmount);
    }

    /// Defines an action that can be taken in withdrawal tests.
    /// @param actionKind The kind of action to take. True for deposit, false
    /// for withdraw.
    /// @param alice The address taking action.
    /// @param token The token being deposited/withdrawn.
    /// @param vaultId The vault being deposited/withdrawn from.
    /// @param amount The amount being deposited/withdrawn. `uint248` is used
    /// as a simple hack to avoid dealing with overflows.
    struct Action {
        bool actionKind;
        address alice;
        address token;
        uint256 vaultId;
        uint248 amount;
    }
    /// Arbitrary interleavings of deposits and withdrawals should work across
    /// many depositors, tokens, and vaults.

    function testWithdrawMany(Action[] memory actions) external {
        vm.assume(actions.length > 0);
        for (uint256 i = 0; i < actions.length; i++) {
            // Deposit and withdrawal amounts must be positive.
            vm.assume(actions[i].amount > 0);
            // Avoid errors from attempting to etch precompiles.
            vm.assume(uint160(actions[i].token) < 1 || 10 < uint160(actions[i].token));
            // Avoid errors from attempting to etch the orderbook.
            vm.assume(actions[i].token != address(orderbook));
        }
        Action memory action;
        for (uint256 i = 0; i < actions.length; i++) {
            vm.etch(action.token, REVERTING_MOCK_BYTECODE);
            action = actions[i];
            uint256 balance = orderbook.vaultBalance(action.alice, action.token, action.vaultId);

            vm.prank(action.alice);
            if (action.actionKind) {
                vm.mockCall(
                    action.token,
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, action.alice, address(orderbook), uint256(action.amount)
                    ),
                    abi.encode(true)
                );
                vm.expectEmit(false, false, false, true);
                emit Deposit(action.alice, action.token, action.vaultId, action.amount);
                orderbook.deposit(action.token, action.vaultId, action.amount);
                assertEq(
                    orderbook.vaultBalance(action.alice, action.token, action.vaultId),
                    balance + action.amount,
                    "vault balance on deposit"
                );
            } else {
                uint256 expectedActualAmount = balance.min(uint256(action.amount));
                vm.mockCall(
                    action.token,
                    abi.encodeWithSelector(IERC20.transfer.selector, action.alice, expectedActualAmount),
                    abi.encode(true)
                );
                if (expectedActualAmount > 0) {
                    vm.expectEmit(false, false, false, true);
                    emit Withdraw(action.alice, action.token, action.vaultId, action.amount, expectedActualAmount);
                }
                orderbook.withdraw(action.token, action.vaultId, action.amount);
                assertEq(
                    orderbook.vaultBalance(action.alice, action.token, action.vaultId),
                    balance - expectedActualAmount,
                    "vault balance on withdraw"
                );
            }
        }
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
