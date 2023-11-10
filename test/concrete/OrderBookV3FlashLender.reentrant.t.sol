// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {IOrderBookV3, OrderConfigV2, Order} from "src/interface/unstable/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter/src/interface/unstable/IParserV1.sol";
import {IERC3156FlashBorrower} from "src/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";

/// @title OrderBookV3FlashLenderReentrant
/// Test that flash borrowers can reenter the orderbook, which is necessary for
/// trading etc. against it while the loan is active.
contract OrderBookV3FlashLenderReentrant is OrderBookExternalRealTest {
    function checkFlashLoanNotRevert(Reenteroor borrower, bytes memory encodedCall, uint256 loanAmount) internal {
        borrower.reenterWith(encodedCall);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(borrower), loanAmount),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.approve.selector, address(iOrderbook), loanAmount),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, borrower, address(iOrderbook), loanAmount),
            abi.encode(true)
        );

        // Create a flash loan.
        iOrderbook.flashLoan(IERC3156FlashBorrower(address(borrower)), address(iToken0), loanAmount, "");
    }

    /// Can reenter and read vault balances from within a flash loan.
    function testReenterReadVaultBalances(uint256 vaultId, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV3.vaultBalance.selector, address(borrower), address(iToken0), vaultId),
            loanAmount
        );
    }

    /// Can reenter and check if an order exists from within a flash loan.
    function testReenterCheckOrderExists(bytes32 orderHash, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV3.orderExists.selector, orderHash), loanAmount
        );
    }

    /// Can reenter and deposit from within a flash loan.
    function testReenterDeposit(uint256 vaultId, uint256 loanAmount, uint256 depositAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        depositAmount = bound(depositAmount, 1, type(uint256).max);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, borrower, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV3.deposit.selector, address(iToken0), vaultId, depositAmount),
            loanAmount
        );
    }

    /// Can reenter and withdraw from within a flash loan.
    function testReenterWithdraw(uint256 vaultId, uint256 loanAmount, uint256 withdrawAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        withdrawAmount = bound(withdrawAmount, 1, type(uint256).max);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(borrower), withdrawAmount),
            abi.encode(true)
        );
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV3.withdraw.selector, address(iToken0), vaultId, withdrawAmount),
            loanAmount
        );
    }

    /// Can reenter and add an order from within a flash loan.
    function testReenterAddOrder(uint256 loanAmount, OrderConfigV2 memory config) external {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iDeployer)).parse("_ _:max-int-value() 1e18;:;");
        config.evaluableConfig.bytecode = bytecode;
        config.evaluableConfig.constants = constants;
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(borrower, abi.encodeWithSelector(IOrderBookV3.addOrder.selector, config), loanAmount);
    }

    /// Can reenter and remove an order from within a flash loan.
    function testReenterRemoveOrder(uint256 loanAmount, Order memory order) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        order.owner = address(borrower);
        checkFlashLoanNotRevert(borrower, abi.encodeWithSelector(IOrderBookV3.removeOrder.selector, order), loanAmount);
    }
}
