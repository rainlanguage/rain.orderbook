// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";

import {OrderBookExternalMockTest, REVERTING_MOCK_BYTECODE} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {Reenteroor, IERC20} from "test/util/concrete/Reenteroor.sol";
import {TaskV1, EvaluableV3, SignedContextV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

/// @title OrderBookWithdrawTest
/// Tests withdrawing from the order book.
contract OrderBookWithdrawTest is OrderBookExternalMockTest {
    using Math for uint256;

    /// Withdrawing a zero target amount should revert.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawZero(address alice, address token, uint256 vaultId) external {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(ZeroWithdrawTargetAmount.selector, alice, token, vaultId));
        iOrderbook.withdraw2(token, vaultId, 0, new TaskV1[](0));
    }

    /// Withdrawing a non-zero amount from an empty vault should be a noop.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawEmptyVault(address alice, address token, uint256 vaultId, uint256 amount) external {
        vm.assume(amount > 0);
        vm.prank(alice);
        vm.record();
        vm.recordLogs();
        iOrderbook.withdraw2(token, vaultId, amount, new TaskV1[](0));
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // Zero logs because nothing happened.
        assertEq(vm.getRecordedLogs().length, 0, "logs");
        // - reentrancy guard x3
        // - vault balance x1
        assertEq(reads.length, 4, "reads");
        // - reentrancy guard x2
        assertEq(writes.length, 2, "writes");
    }

    /// Withdrawing the full amount from a vault should delete the vault.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawFullVault(address alice, uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount)
        external
    {
        vm.assume(depositAmount > 0);
        vm.assume(withdrawAmount >= depositAmount);
        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new TaskV1[](0));
        assertEq(iOrderbook.vaultBalance(address(alice), address(iToken0), vaultId), depositAmount);

        vm.prank(alice);
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, depositAmount), abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit Withdraw(alice, address(iToken0), vaultId, withdrawAmount, depositAmount);
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, new TaskV1[](0));
        assertEq(iOrderbook.vaultBalance(address(alice), address(iToken0), vaultId), 0);
    }

    /// Withdrawing a partial amount from a vault should reduce the vault balance.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawPartialVault(address alice, uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount)
        external
    {
        vm.assume(depositAmount > 0);
        vm.assume(withdrawAmount > 0);
        vm.assume(withdrawAmount < depositAmount);
        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new TaskV1[](0));
        assertEq(iOrderbook.vaultBalance(address(alice), address(iToken0), vaultId), depositAmount);

        vm.prank(alice);
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount), abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        // The full withdraw amount is possible as it's only a partial withdraw.
        emit Withdraw(alice, address(iToken0), vaultId, withdrawAmount, withdrawAmount);
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, new TaskV1[](0));
        // The vault balance is reduced by the withdraw amount.
        assertEq(iOrderbook.vaultBalance(address(alice), address(iToken0), vaultId), depositAmount - withdrawAmount);
    }

    /// Any failure in the withdrawal should revert the entire transaction.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawFailure(address alice, uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount)
        external
    {
        vm.assume(depositAmount > 0);
        vm.assume(withdrawAmount > 0);
        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new TaskV1[](0));
        assertEq(iOrderbook.vaultBalance(address(alice), address(iToken0), vaultId), depositAmount);

        // The token contract always reverts when not mocked.
        vm.prank(alice);
        vm.expectRevert("SafeERC20: low-level call failed");
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, new TaskV1[](0));

        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount.min(depositAmount)),
            abi.encode(false)
        );
        vm.expectRevert("SafeERC20: ERC20 operation did not succeed");
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, new TaskV1[](0));
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
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawMany(Action[] memory actions) external {
        vm.assume(actions.length > 0);
        for (uint256 i = 0; i < actions.length; i++) {
            // Deposit and withdrawal amounts must be positive.
            actions[i].amount = uint248(bound(actions[i].amount, 1, type(uint248).max));
            // Ensure the token doesn't hit some known address and cause bad
            // etching.
            actions[i].token = address(uint160(uint256(keccak256(abi.encodePacked(actions[i].token)))));
        }
        Action memory action;
        for (uint256 i = 0; i < actions.length; i++) {
            vm.etch(action.token, REVERTING_MOCK_BYTECODE);
            action = actions[i];
            uint256 balance = iOrderbook.vaultBalance(action.alice, action.token, action.vaultId);

            vm.prank(action.alice);
            if (action.actionKind) {
                vm.mockCall(
                    action.token,
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, action.alice, address(iOrderbook), uint256(action.amount)
                    ),
                    abi.encode(true)
                );
                vm.expectEmit(false, false, false, true);
                emit Deposit(action.alice, action.token, action.vaultId, action.amount);
                iOrderbook.deposit2(action.token, action.vaultId, action.amount, new TaskV1[](0));
                assertEq(
                    iOrderbook.vaultBalance(action.alice, action.token, action.vaultId),
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
                iOrderbook.withdraw2(action.token, action.vaultId, action.amount, new TaskV1[](0));
                assertEq(
                    iOrderbook.vaultBalance(action.alice, action.token, action.vaultId),
                    balance - expectedActualAmount,
                    "vault balance on withdraw"
                );
            }
        }
    }
}
