// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";

import "test/util/abstract/OrderBookTest.sol";
import "test/util/concrete/Reenteroor.sol";

/// @title OrderBookDepositTest
/// Tests depositing to an order book without any trades.
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

    /// It must always be possible to deposit to a vault, assuming it does not
    /// overflow.
    //solhint-disable-next-line func-name-mixedcase
    function invariant_canAlwaysDeposit() external {
        uint256 vaultId = uint256(keccak256(abi.encodePacked(block.timestamp)));
        uint256 amount = uint256(keccak256(abi.encodePacked(vaultId)));

        vm.mockCall(
            address(token0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(orderbook), amount),
            abi.encode(true)
        );
        orderbook.deposit(address(token0), vaultId, amount);
        assertEq(orderbook.vaultBalance(address(this), address(token0), vaultId), amount);
    }
}
