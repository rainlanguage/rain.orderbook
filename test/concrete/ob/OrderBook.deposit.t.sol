// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {stdError} from "forge-std/Test.sol";
import {REVERTING_MOCK_BYTECODE, CONSOLE_ADDRESS} from "test/util/lib/LibTestConstants.sol";
import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {TaskV2, EvaluableV4, IOrderBookV5} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title OrderBookDepositTest
/// Tests depositing to an order book.
contract OrderBookDepositTest is OrderBookExternalMockTest {
    using LibDecimalFloat for Float;

    /// Tests that we can deposit some amount and view the new vault balance.
    /// forge-config: default.fuzz.runs = 100
    function testDepositSimple(address depositor, bytes32 vaultId, uint256 amount18) external {
        vm.assume(amount18 > 0);
        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount18),
            abi.encode(true)
        );

        Float memory amount = LibDecimalFloat.fromFixedDecimalLosslessMem(amount18, 18);

        iOrderbook.deposit3(address(iToken0), vaultId, amount, new TaskV2[](0));
        assertTrue(iOrderbook.vaultBalance2(depositor, address(iToken0), vaultId).eq(amount));
    }

    /// Depositing zero should revert.
    /// forge-config: default.fuzz.runs = 100
    function testDepositZero(address depositor, bytes32 vaultId) external {
        vm.prank(depositor);
        vm.expectRevert(
            abi.encodeWithSelector(
                IOrderBookV5.ZeroDepositAmount.selector, address(depositor), address(iToken0), vaultId
            )
        );
        iOrderbook.deposit3(address(iToken0), vaultId, Float(0, 0), new TaskV2[](0));
    }

    /// Test a warm deposit, which is the best case scenario for gas. In this
    /// case the storage backing the vault balance is already warm so an
    /// additional deposit gets a much cheaper sstore.
    function testDepositGas00() external {
        vm.pauseGasMetering();
        // warm up storage
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), 1),
            abi.encode(true)
        );
        iOrderbook.deposit3(address(iToken0), 0, Float(1, 0), new TaskV2[](0));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), 1),
            abi.encode(true)
        );
        vm.resumeGasMetering();
        iOrderbook.deposit3(address(iToken0), 0, Float(1, 0), new TaskV2[](0));
    }

    /// Test a cold deposit, which is the worst case scenario for gas. In this
    /// case the storage backing the vault balance is cold so the first deposit
    /// gets a much more expensive sstore. Unfortunately this is the case for
    /// most deposits.
    function testDepositGas01() external {
        vm.pauseGasMetering();
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), 1),
            abi.encode(true)
        );
        vm.resumeGasMetering();
        iOrderbook.deposit3(address(iToken0), 0, Float(1, 0), new TaskV2[](0));
    }

    /// Any failure in the deposit should revert the entire transaction.
    /// forge-config: default.fuzz.runs = 100
    function testDepositFail(address depositor, bytes32 vaultId, uint256 amount18) external {
        vm.assume(amount18 > 0);
        Float memory amount = LibDecimalFloat.fromFixedDecimalLosslessMem(amount18, 18);

        // The token contract always reverts when not mocked.
        vm.prank(depositor);
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        iOrderbook.deposit3(address(iToken0), vaultId, amount, new TaskV2[](0));

        // Mocking the token to return false should also revert.
        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount18),
            abi.encode(false)
        );
        // This error string appears when the call completes but returns false.
        vm.expectRevert(bytes("SafeERC20: ERC20 operation did not succeed"));
        iOrderbook.deposit3(address(iToken0), vaultId, amount, new TaskV2[](0));
    }

    /// Defines a deposit to be used in testDepositMany.
    /// @param depositor The address of the depositor.
    /// @param token The address of the token to deposit.
    /// @param vaultId The vaultId to deposit to.
    /// @param amount The amount to deposit.
    struct Action {
        address depositor;
        address token;
        bytes32 vaultId;
        Float amount;
    }

    /// Any combination of depositors, tokens, vaults, amounts should not cause
    /// collisions or other illogical outcomes.
    /// forge-config: default.fuzz.runs = 100
    function testDepositMany(Action[] memory actions) external {
        vm.assume(actions.length > 0);
        for (uint256 i = 0; i < actions.length; i++) {
            // Deposit amounts must be non-zero.
            vm.assume(actions[i].amount.gt(Float(0, 0)));
            // Avoid errors from attempting to etch precompiles.
            vm.assume(uint160(actions[i].token) < 1 || 10 < uint160(actions[i].token));
            // Avoid errors from attempting to etch the orderbook.
            vm.assume(actions[i].token != address(iOrderbook));
            // Avoid errors from attempting to etch test harness internals.
            vm.assume(actions[i].token != address(CONSOLE_ADDRESS));
            vm.assume(actions[i].token != address(vm));
        }

        for (uint256 i = 0; i < actions.length; i++) {
            vm.etch(actions[i].token, REVERTING_MOCK_BYTECODE);
            Float memory vaultBalanceBefore =
                iOrderbook.vaultBalance2(actions[i].depositor, actions[i].token, actions[i].vaultId);
            vm.prank(actions[i].depositor);
            vm.mockCall(
                actions[i].token,
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector,
                    actions[i].depositor,
                    address(iOrderbook),
                    LibDecimalFloat.toFixedDecimalLossless(actions[i].amount, 18)
                ),
                abi.encode(true)
            );
            vm.expectEmit(false, false, false, true);
            emit DepositV2(
                actions[i].depositor,
                actions[i].token,
                actions[i].vaultId,
                LibDecimalFloat.toFixedDecimalLossless(actions[i].amount, 18)
            );
            vm.record();
            vm.recordLogs();
            iOrderbook.deposit3(actions[i].token, actions[i].vaultId, actions[i].amount, new TaskV2[](0));
            (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
            assertEq(vm.getRecordedLogs().length, 1, "logs");
            // - reentrancy guard x3
            // - vault balance x2
            assertEq(reads.length, 5, "reads");
            // - reentrancy guard x2
            // - vault balance x1
            assertEq(writes.length, 3, "writes");
            assertTrue(
                iOrderbook.vaultBalance2(actions[i].depositor, actions[i].token, actions[i].vaultId).eq(
                    actions[i].amount.add(vaultBalanceBefore)
                ),
                "vault balance"
            );
        }
    }

    /// Depositing should emit an event with the sender and all deposit details.
    /// forge-config: default.fuzz.runs = 100
    function testDepositEvent(address depositor, bytes32 vaultId, uint256 amount18) external {
        vm.assume(amount18 > 0);
        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount18),
            abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit DepositV2(depositor, address(iToken0), vaultId, amount18);
        Float memory amount = LibDecimalFloat.fromFixedDecimalLosslessMem(amount18, 18);
        iOrderbook.deposit3(address(iToken0), vaultId, amount, new TaskV2[](0));
    }

    /// Depositing should NOT allow reentrancy.
    /// forge-config: default.fuzz.runs = 100
    function testDepositReentrancy(
        address depositor,
        bytes32 vaultId,
        uint256 amount18,
        address reToken,
        bytes32 reVaultId,
        uint256 reAmount18
    ) external {
        vm.assume(amount18 != 0);
        vm.assume(reAmount18 != 0);
        vm.prank(depositor);
        Reenteroor reenteroor = new Reenteroor();
        Float memory amount = LibDecimalFloat.fromFixedDecimalLosslessMem(amount18, 18);
        Float memory reAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(reAmount18, 18);
        reenteroor.reenterWith(
            abi.encodeWithSelector(IOrderBookV5.deposit3.selector, reToken, reVaultId, reAmount, new TaskV2[](0))
        );
        vm.expectRevert(bytes("ReentrancyGuard: reentrant call"));
        iOrderbook.deposit3(address(reenteroor), vaultId, amount, new TaskV2[](0));
    }

    /// Vault balances MUST NOT silently overflow.
    /// forge-config: default.fuzz.runs = 100
    function testDepositOverflow(address depositor, bytes32 vaultId, uint256 amountOne18, uint256 amountTwo18)
        external
    {
        amountOne18 = bound(amountOne18, type(uint128).max, type(uint256).max);
        amountTwo18 = bound(amountTwo18, type(uint128).max, type(uint256).max);

        Float memory amountOne = LibDecimalFloat.fromFixedDecimalLosslessMem(amountOne18, 18);
        Float memory amountTwo = LibDecimalFloat.fromFixedDecimalLosslessMem(amountTwo18, 18);

        bool didOverflow = false;
        assembly {
            didOverflow := lt(add(amountOne18, amountTwo18), amountOne18)
        }
        vm.assume(didOverflow == true);

        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amountOne18),
            abi.encode(true)
        );
        iOrderbook.deposit3(address(iToken0), vaultId, amountOne, new TaskV2[](0));

        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amountTwo18),
            abi.encode(true)
        );
        vm.expectRevert(stdError.arithmeticError);
        iOrderbook.deposit3(address(iToken0), vaultId, amountTwo, new TaskV2[](0));
    }

    // Invariant testing doesn't seem to work currently.
    // https://github.com/foundry-rs/foundry/issues/4656
    // /// It must always be possible to deposit to a vault, assuming it does not
    // /// overflow.
    // //solhint-disable-next-line func-name-mixedcase
    // function invariant_CanAlwaysDeposit() external {
    //     uint256 vaultId = uint256(keccak256(abi.encodePacked(block.timestamp)));
    //     uint256 amount = uint256(keccak256(abi.encodePacked(vaultId)));

    //     vm.mockCall(
    //         address(token0),
    //         abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(orderbook), amount),
    //         abi.encode(true)
    //     );
    //     orderbook.deposit(address(token0), vaultId, amount);
    //     assertEq(orderbook.vaultBalance(address(this), address(token0), vaultId), amount);
    // }
}
