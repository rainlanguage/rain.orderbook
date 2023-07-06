// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookDepositTest
/// Tests depositing to an order book.
contract OrderBookDepositTest is OrderBookTest {
    /// Tests that we can deposit some amount and view the new vault balance.
    function testDepositSimple(address depositor, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);
        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(true)
        );

        orderbook.deposit(address(token0), vaultId, amount);
        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), amount);
    }

    /// Depositing zero should revert.
    function testDepositZero(address depositor, uint256 vaultId) external {
        vm.prank(depositor);
        vm.expectRevert(
            abi.encodeWithSelector(
                IOrderBookV3.ZeroDepositAmount.selector, address(depositor), address(token0), vaultId
            )
        );
        orderbook.deposit(address(token0), vaultId, 0);
    }

    /// Test a warm deposit, which is the best case scenario for gas. In this
    /// case the storage backing the vault balance is already warm so an
    /// additional deposit gets a much cheaper sstore.
    function testDepositGas00() external {
        vm.pauseGasMetering();
        // warm up storage
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(orderbook), 1),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), 0, 1);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(orderbook), 1),
            abi.encode(true)
        );
        vm.resumeGasMetering();
        orderbook.deposit(address(token0), 0, 1);
    }

    /// Test a cold deposit, which is the worst case scenario for gas. In this
    /// case the storage backing the vault balance is cold so the first deposit
    /// gets a much more expensive sstore. Unfortunately this is the case for
    /// most deposits.
    function testDepositGas01() external {
        vm.pauseGasMetering();
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(orderbook), 1),
            abi.encode(true)
        );
        vm.resumeGasMetering();
        orderbook.deposit(address(token0), 0, 1);
    }

    /// Any failure in the deposit should revert the entire transaction.
    function testDepositFail(address depositor, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);

        // The token contract always reverts when not mocked.
        vm.prank(depositor);
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        orderbook.deposit(address(token0), vaultId, amount);

        // Mocking the token to return false should also revert.
        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(false)
        );
        // This error string appears when the call completes but returns false.
        vm.expectRevert(bytes("SafeERC20: ERC20 operation did not succeed"));
        orderbook.deposit(address(token0), vaultId, amount);
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
            vm.assume(actions[i].token != address(orderbook));
            // vm.assume(actions[i].token != address(console));
        }

        for (uint256 i = 0; i < actions.length; i++) {
            vm.etch(actions[i].token, REVERTING_MOCK_BYTECODE);
            uint256 vaultBalanceBefore =
                orderbook.vaultBalance(actions[i].depositor, actions[i].token, actions[i].vaultId);
            vm.prank(actions[i].depositor);
            vm.mockCall(
                actions[i].token,
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector, actions[i].depositor, address(orderbook), uint256(actions[i].amount)
                ),
                abi.encode(true)
            );
            orderbook.deposit(actions[i].token, actions[i].vaultId, actions[i].amount);
            assertEq(
                orderbook.vaultBalance(actions[i].depositor, actions[i].token, actions[i].vaultId),
                actions[i].amount + vaultBalanceBefore,
                "vault balance"
            );
        }
    }

    /// Multiple deposits should be additive.
    function testDepositMultiple(address depositor, uint256 vaultId, uint256[] memory amounts) external {
        uint256 totalAmount = 0;
        uint256 amount;
        for (uint256 i = 0; i < amounts.length; i++) {
            amount = amounts[i] % type(uint248).max;
            vm.assume(amount != 0);
            totalAmount += amount;
            vm.prank(depositor);
            vm.mockCall(
                address(token0),
                abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
                abi.encode(true)
            );
            orderbook.deposit(address(token0), vaultId, amount);
            assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), totalAmount);
        }
    }

    /// Depositing under different tokens should not affect each other even if
    /// the vaultId is the same.
    function testMultiTokenCollision(address depositor, uint256 vaultId, uint256 amountOne, uint256 amountTwo)
        external
    {
        vm.assume(amountOne != 0);
        vm.assume(amountTwo != 0);

        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amountOne),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, amountOne);
        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), amountOne);

        vm.prank(depositor);
        vm.mockCall(
            address(token1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amountTwo),
            abi.encode(true)
        );
        orderbook.deposit(address(token1), vaultId, amountTwo);
        assertEq(orderbook.vaultBalance(depositor, address(token1), vaultId), amountTwo);
    }

    /// Depositing under different vaults should not affect each other even if
    /// the token is the same.
    function testMultiVaultCollision(
        address depositor,
        uint256 vaultIdOne,
        uint256 vaultIdTwo,
        uint256 amountOne,
        uint256 amountTwo
    ) external {
        vm.assume(amountOne != 0);
        vm.assume(amountTwo != 0);
        vm.assume(vaultIdOne != vaultIdTwo);

        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amountOne),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultIdOne, amountOne);
        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultIdOne), amountOne);

        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amountTwo),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultIdTwo, amountTwo);
        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultIdTwo), amountTwo);
    }

    /// Two different depositors should not affect each other even if the token
    /// and vaultId are the same.
    function testMultiDepositorCollision(
        address alice,
        address bob,
        uint256 vaultId,
        uint256 amountAlice,
        uint256 amountBob
    ) external {
        vm.assume(amountAlice != 0);
        vm.assume(amountBob != 0);
        vm.assume(alice != bob);

        vm.prank(alice);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(orderbook), amountAlice),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, amountAlice);
        assertEq(orderbook.vaultBalance(alice, address(token0), vaultId), amountAlice);

        vm.prank(bob);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(orderbook), amountBob),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, amountBob);
        assertEq(orderbook.vaultBalance(bob, address(token0), vaultId), amountBob);
    }

    /// Depositing should emit an event with the sender and all deposit details.
    function testDepositEvent(address depositor, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);
        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit Deposit(depositor, address(token0), vaultId, amount);
        orderbook.deposit(address(token0), vaultId, amount);
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
        orderbook.deposit(address(reenteroor), vaultId, amount);
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
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amountOne),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, amountOne);

        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amountTwo),
            abi.encode(true)
        );
        vm.expectRevert(stdError.arithmeticError);
        orderbook.deposit(address(token0), vaultId, amountTwo);
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
