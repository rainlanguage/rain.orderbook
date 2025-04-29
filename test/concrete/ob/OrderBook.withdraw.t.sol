// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";

import {OrderBookExternalMockTest, REVERTING_MOCK_BYTECODE} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {Reenteroor, IERC20} from "test/util/concrete/Reenteroor.sol";
import {TaskV2} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibDecimalFloatImplementation} from "rain.math.float/lib/implementation/LibDecimalFloatImplementation.sol";

/// @title OrderBookWithdrawTest
/// Tests withdrawing from the order book.
contract OrderBookWithdrawTest is OrderBookExternalMockTest {
    using Math for uint256;

    using LibDecimalFloat for Float;
    using LibDecimalFloatImplementation for Float;

    mapping(address => bool) internal sHasDeposit;

    /// Withdrawing a zero target amount should revert.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawZero(address alice, address token, bytes32 vaultId) external {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(ZeroWithdrawTargetAmount.selector, alice, token, vaultId));
        iOrderbook.withdraw3(token, vaultId, Float.wrap(0), new TaskV2[](0));
    }

    /// Withdrawing a non-zero amount from an empty vault should be a noop.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawEmptyVault(address alice, address token, bytes32 vaultId, uint256 amount18) external {
        amount18 = bound(amount18, 1, uint256(int256(type(int224).max)));
        vm.prank(alice);
        Float amount = LibDecimalFloat.fromFixedDecimalLosslessPacked(amount18, 18);
        vm.mockCall(token, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));
        vm.expectEmit(false, false, false, true);
        emit WithdrawV2(alice, token, vaultId, amount, Float.wrap(0), 0);
        vm.record();
        iOrderbook.withdraw3(token, vaultId, amount, new TaskV2[](0));
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        assertEq(reads.length, 8, "reads");
        assertEq(writes.length, 4, "writes");
    }

    /// Withdrawing the full amount from a vault should delete the vault.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawFullVault(address alice, bytes32 vaultId, uint256 depositAmount18, uint256 withdrawAmount18)
        external
    {
        depositAmount18 = bound(depositAmount18, 1, type(uint224).max / 10);
        withdrawAmount18 = bound(withdrawAmount18, depositAmount18, type(uint224).max / 10);
        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount18),
            abi.encode(true)
        );

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        iOrderbook.deposit3(address(iToken0), vaultId, depositAmount, new TaskV2[](0));
        assertTrue(iOrderbook.vaultBalance2(address(alice), address(iToken0), vaultId).eq(depositAmount));

        vm.prank(alice);
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, depositAmount18), abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit WithdrawV2(alice, address(iToken0), vaultId, withdrawAmount, depositAmount, depositAmount18);
        iOrderbook.withdraw3(address(iToken0), vaultId, withdrawAmount, new TaskV2[](0));
        assertTrue(iOrderbook.vaultBalance2(address(alice), address(iToken0), vaultId).isZero(), "vault balance");
    }

    /// Withdrawing a partial amount from a vault should reduce the vault balance.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawPartialVault(address alice, bytes32 vaultId, uint256 depositAmount18, uint256 withdrawAmount18)
        external
    {
        depositAmount18 = bound(depositAmount18, 2, type(uint224).max / 10);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18 - 1);
        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount18),
            abi.encode(true)
        );

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        iOrderbook.deposit3(address(iToken0), vaultId, depositAmount, new TaskV2[](0));
        assertTrue(iOrderbook.vaultBalance2(address(alice), address(iToken0), vaultId).eq(depositAmount));

        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount18),
            abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        // The full withdraw amount is possible as it's only a partial withdraw.
        emit WithdrawV2(alice, address(iToken0), vaultId, withdrawAmount, withdrawAmount, withdrawAmount18);
        iOrderbook.withdraw3(address(iToken0), vaultId, withdrawAmount, new TaskV2[](0));
        // The vault balance is reduced by the withdraw amount.
        assertTrue(
            iOrderbook.vaultBalance2(address(alice), address(iToken0), vaultId).eq(depositAmount.sub(withdrawAmount))
        );
    }

    /// Any failure in the withdrawal should revert the entire transaction.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawFailure(address alice, bytes32 vaultId, uint256 depositAmount18, uint256 withdrawAmount18)
        external
    {
        depositAmount18 = bound(depositAmount18, 1, type(uint224).max / 10);
        withdrawAmount18 = bound(withdrawAmount18, 1, type(uint224).max / 10);
        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount18),
            abi.encode(true)
        );
        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);
        iOrderbook.deposit3(address(iToken0), vaultId, depositAmount, new TaskV2[](0));
        assertTrue(iOrderbook.vaultBalance2(address(alice), address(iToken0), vaultId).eq(depositAmount));

        // The token contract always reverts when not mocked.
        vm.prank(alice);
        vm.expectRevert("SafeERC20: low-level call failed");
        iOrderbook.withdraw3(address(iToken0), vaultId, withdrawAmount, new TaskV2[](0));

        vm.prank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount18.min(depositAmount18)),
            abi.encode(false)
        );
        vm.expectRevert("SafeERC20: ERC20 operation did not succeed");
        iOrderbook.withdraw3(address(iToken0), vaultId, withdrawAmount, new TaskV2[](0));
    }

    /// Defines an action that can be taken in withdrawal tests.
    /// @param actionKind The kind of action to take. True for deposit, false
    /// for withdraw.
    /// @param alice The address taking action.
    /// @param token The token being deposited/withdrawn.
    /// @param vaultId The vault being deposited/withdrawn from.
    /// @param amount The amount being deposited/withdrawn.
    struct Action {
        bool actionKind;
        address alice;
        address token;
        bytes32 vaultId;
        uint256 amount;
        Float amountFloat;
        uint256 pairedWith;
        bool fresh;
    }

    /// Arbitrary interleavings of deposits and withdrawals should work across
    /// many depositors, tokens, and vaults.
    /// forge-config: default.fuzz.runs = 100
    function testWithdrawMany(Action[] memory actions) external {
        vm.assume(actions.length > 0);
        for (uint256 i = 0; i < actions.length; i++) {
            // Deposit and withdrawal amounts must be positive.
            actions[i].amount = bound(actions[i].amount, 1, type(uint224).max / 10);
            // Ensure the token doesn't hit some known address and cause bad
            // etching.
            actions[i].token = address(uint160(uint256(keccak256(abi.encodePacked(actions[i].token)))));
            actions[i].amountFloat = LibDecimalFloat.fromFixedDecimalLosslessPacked(actions[i].amount, 18);

            if (!actions[i].fresh && i > 0) {
                actions[i].pairedWith = bound(actions[i].pairedWith, 0, i - 1);
                actions[i].alice = actions[actions[i].pairedWith].alice;
                actions[i].vaultId = actions[actions[i].pairedWith].vaultId;
                actions[i].token = actions[actions[i].pairedWith].token;
            }
        }
        Action memory action;
        for (uint256 i = 0; i < actions.length; i++) {
            vm.etch(action.token, REVERTING_MOCK_BYTECODE);
            action = actions[i];
            Float balance = iOrderbook.vaultBalance2(action.alice, action.token, action.vaultId);

            vm.prank(action.alice);
            if (action.actionKind || !sHasDeposit[action.token]) {
                sHasDeposit[action.token] = true;
                vm.mockCall(
                    action.token,
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, action.alice, address(iOrderbook), uint256(action.amount)
                    ),
                    abi.encode(true)
                );
                vm.mockCall(
                    action.token, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18))
                );

                vm.expectEmit(false, false, false, true);
                emit DepositV2(action.alice, action.token, action.vaultId, action.amount);
                iOrderbook.deposit3(action.token, action.vaultId, action.amountFloat, new TaskV2[](0));
                assertTrue(
                    iOrderbook.vaultBalance2(action.alice, action.token, action.vaultId).eq(
                        balance.add(action.amountFloat)
                    ),
                    "vault balance on deposit"
                );
            } else {
                Float expectedActualAmount = action.amountFloat.min(balance);
                uint256 expectedActualAmount18 = expectedActualAmount.toFixedDecimalLossless(18);
                vm.mockCall(
                    action.token,
                    abi.encodeWithSelector(IERC20.transfer.selector, action.alice, expectedActualAmount18),
                    abi.encode(true)
                );
                vm.mockCall(
                    action.token, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18))
                );

                if (expectedActualAmount.gt(Float.wrap(0))) {
                    vm.expectEmit(false, false, false, true);
                    emit WithdrawV2(
                        action.alice,
                        action.token,
                        action.vaultId,
                        action.amountFloat,
                        expectedActualAmount,
                        expectedActualAmount18
                    );
                }
                iOrderbook.withdraw3(action.token, action.vaultId, action.amountFloat, new TaskV2[](0));
                assertTrue(
                    iOrderbook.vaultBalance2(action.alice, action.token, action.vaultId).eq(
                        balance.sub(expectedActualAmount)
                    ),
                    "vault balance on withdraw"
                );
            }
        }
    }
}
