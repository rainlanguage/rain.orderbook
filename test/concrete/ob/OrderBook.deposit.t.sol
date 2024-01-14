// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/forge-std/src/Test.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/abstract/OrderBookExternalMockTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookDepositTest
/// Tests depositing to an order book.
contract OrderBookDepositTest is OrderBookExternalMockTest {
    /// Tests that we can deposit some amount and view the new vault balance.
    function testDepositSimple(address depositor, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);
        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(true)
        );

        iOrderbook.deposit(address(iToken0), vaultId, amount);
        assertEq(iOrderbook.vaultBalance(depositor, address(iToken0), vaultId), amount);
    }

    /// Depositing zero should revert.
    function testDepositZero(address depositor, uint256 vaultId) external {
        vm.prank(depositor);
        vm.expectRevert(
            abi.encodeWithSelector(
                IOrderBookV3.ZeroDepositAmount.selector, address(depositor), address(iToken0), vaultId
            )
        );
        iOrderbook.deposit(address(iToken0), vaultId, 0);
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
        iOrderbook.deposit(address(iToken0), 0, 1);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), 1),
            abi.encode(true)
        );
        vm.resumeGasMetering();
        iOrderbook.deposit(address(iToken0), 0, 1);
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
        iOrderbook.deposit(address(iToken0), 0, 1);
    }

    /// Any failure in the deposit should revert the entire transaction.
    function testDepositFail(address depositor, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);

        // The token contract always reverts when not mocked.
        vm.prank(depositor);
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        iOrderbook.deposit(address(iToken0), vaultId, amount);

        // Mocking the token to return false should also revert.
        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(false)
        );
        // This error string appears when the call completes but returns false.
        vm.expectRevert(bytes("SafeERC20: ERC20 operation did not succeed"));
        iOrderbook.deposit(address(iToken0), vaultId, amount);
    }

    /// Defines a deposit to be used in testDepositMany.
    /// @param depositor The address of the depositor.
    /// @param token The address of the token to deposit.
    /// @param vaultId The vaultId to deposit to.
    /// @param amount The amount to deposit. `uint248` is used to avoid overflow.
    struct Action {
        address depositor;
        address token;
        uint256 vaultId;
        uint248 amount;
    }

    /// Any combination of depositors, tokens, vaults, amounts should not cause
    /// collisions or other illogical outcomes.
    function testDepositMany(Action[] memory actions) external {
        vm.assume(actions.length > 0);
        for (uint256 i = 0; i < actions.length; i++) {
            // Deposit amounts must be non-zero.
            vm.assume(actions[i].amount != 0);
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
            uint256 vaultBalanceBefore =
                iOrderbook.vaultBalance(actions[i].depositor, actions[i].token, actions[i].vaultId);
            vm.prank(actions[i].depositor);
            vm.mockCall(
                actions[i].token,
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector, actions[i].depositor, address(iOrderbook), uint256(actions[i].amount)
                ),
                abi.encode(true)
            );
            vm.expectEmit(false, false, false, true);
            emit Deposit(actions[i].depositor, actions[i].token, actions[i].vaultId, uint256(actions[i].amount));
            vm.record();
            vm.recordLogs();
            iOrderbook.deposit(actions[i].token, actions[i].vaultId, actions[i].amount);
            (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
            assertEq(vm.getRecordedLogs().length, 1, "logs");
            // - reentrancy guard x3
            // - vault balance x2
            assertEq(reads.length, 5, "reads");
            // - reentrancy guard x2
            // - vault balance x1
            assertEq(writes.length, 3, "writes");
            assertEq(
                iOrderbook.vaultBalance(actions[i].depositor, actions[i].token, actions[i].vaultId),
                actions[i].amount + vaultBalanceBefore,
                "vault balance"
            );
        }
    }

    /// Depositing should emit an event with the sender and all deposit details.
    function testDepositEvent(address depositor, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);
        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit Deposit(depositor, address(iToken0), vaultId, amount);
        iOrderbook.deposit(address(iToken0), vaultId, amount);
    }

    /// Depositing should NOT allow reentrancy.
    function testDepositReentrancy(
        address depositor,
        uint256 vaultId,
        uint256 amount,
        address reToken,
        uint256 reVaultId,
        uint256 reAmount
    ) external {
        vm.assume(amount != 0);
        vm.assume(reAmount != 0);
        vm.prank(depositor);
        Reenteroor reenteroor = new Reenteroor();
        reenteroor.reenterWith(abi.encodeWithSelector(IOrderBookV3.deposit.selector, reToken, reVaultId, reAmount));
        vm.expectRevert(bytes("ReentrancyGuard: reentrant call"));
        iOrderbook.deposit(address(reenteroor), vaultId, amount);
    }

    /// Vault balances MUST NOT silently overflow.
    function testDepositOverflow(address depositor, uint256 vaultId, uint256 amountOne, uint256 amountTwo) external {
        bool didOverflow = false;
        assembly {
            didOverflow := lt(add(amountOne, amountTwo), amountOne)
        }
        vm.assume(didOverflow == true);

        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amountOne),
            abi.encode(true)
        );
        iOrderbook.deposit(address(iToken0), vaultId, amountOne);

        vm.prank(depositor);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amountTwo),
            abi.encode(true)
        );
        vm.expectRevert(stdError.arithmeticError);
        iOrderbook.deposit(address(iToken0), vaultId, amountTwo);
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
