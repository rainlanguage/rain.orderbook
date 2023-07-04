// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookTest.sol";

/// @title OrderBookDepositTest
/// Tests depositing to an order book without any trades.
contract OrderBookDepositTest is OrderBookTest {
    /// Tests that we can deposit some amount and view the new vault balance.
    function testDepositSimple(address depositor, uint256 vaultId, uint256 amount) external {
        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(true)
        );

        vm.record();
        orderbook.deposit(address(token0), vaultId, amount);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses((address(orderbook)));
        assertEq(reads.length, 5);
        assertEq(writes.length, 3);
        assertEq(writes[0], bytes32(0));
        // assertEq(writes[1], bytes32(0));

        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), amount);
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
        vm.prank(depositor);

        // The token contract always reverts when not mocked.
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        orderbook.deposit(address(token0), vaultId, amount);

        // Mocking the token to return false should also revert.
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(false)
        );
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        orderbook.deposit(address(token0), vaultId, amount);
    }

    /// Multiple deposits should be additive.
    function testDepositMultiple(address depositor, uint256 vaultId, uint256[] memory amounts) external {
        uint256 totalAmount = 0;
        uint256 amount;
        for (uint256 i = 0; i < amounts.length; i++) {
            amount = amounts[i] % type(uint248).max;
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

    /// Depositing should emit an event with the sender and all deposit details.
    function testDepositEvent(address depositor, uint256 vaultId, uint256 amount) external {
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
}
