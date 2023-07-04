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
        orderbook.deposit(DepositConfig(address(token0), vaultId, amount));
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses((address(orderbook)));
        // assertEq(reads.length, 0);
        // assertEq(writes.length, 1);
        // assertEq(writes[0], bytes32(uint256(1)));

        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), amount);
    }

    function testDepositGas00() external {
        vm.pauseGasMetering();
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(orderbook), 1),
            abi.encode(true)
        );
        vm.resumeGasMetering();

        orderbook.deposit(DepositConfig(address(token0), 0, 1));
    }

    /// Any failure in the deposit should revert the entire transaction.
    function testDepositFail(address depositor, uint256 vaultId, uint256 amount) external {
        vm.prank(depositor);

        // The token contract always reverts when not mocked.
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        orderbook.deposit(DepositConfig(address(token0), vaultId, amount));

        // Mocking the token to return false should also revert.
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(false)
        );
        vm.expectRevert(bytes("SafeERC20: low-level call failed"));
        orderbook.deposit(DepositConfig(address(token0), vaultId, amount));
    }

    /// Multiple deposits should be additive.
    function testDepositMultiple(address depositor, uint256 vaultId, uint256[] memory amounts) external {
        uint256 totalAmount;
        for (uint256 i = 0; i < amounts.length; i++) {
            vm.assume(type(uint256).max - totalAmount >= amounts[i]); // Prevent overflow.
            totalAmount += amounts[i];

            vm.prank(depositor);
            vm.mockCall(
                address(token0),
                abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amounts[i]),
                abi.encode(true)
            );
            orderbook.deposit(DepositConfig(address(token0), vaultId, amounts[i]));
            assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), totalAmount);
        }
    }

    /// Depositing should emit an event with the sender and all deposit details.
    function testDepositEvent(address depositor, DepositConfig memory config) external {
        config.token = address(token0);
        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), config.amount),
            abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit Deposit(depositor, config);
        orderbook.deposit(config);
    }
}
