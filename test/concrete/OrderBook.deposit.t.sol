// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookTest.sol";

/// @title OrderBookDepositTest
/// Tests depositing to an order book without any trades.
contract OrderBookDepositTest is OrderBookTest {
    /// Tests that we can deposit some amount.
    function testDeposit(address depositor, uint256 vaultId, uint256 amount) external {
        vm.prank(depositor);
        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(orderbook), amount),
            abi.encode(true)
        );
        orderbook.deposit(DepositConfig(address(token0), vaultId, amount));
        assertEq(orderbook.vaultBalance(depositor, address(token0), vaultId), amount);
    }

    /// Any failure in the deposit should revert the entire transaction.
    function testDepositFail(address depositor, uint256 vaultId, uint256 amount) external {
        vm.prank(depositor);
        vm.expectRevert("foo");
        orderbook.deposit(DepositConfig(address(token0), vaultId, amount));
    }
}
