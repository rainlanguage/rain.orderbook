// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {
    IOrderBookV4,
    OrderConfigV3,
    OrderV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    ClearConfig,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/deprecated/IInterpreterCallerV2.sol";
import {EvaluableV3} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";

/// @title OrderBookV4FlashLenderReentrant
/// Test that flash borrowers can reenter the orderbook, which is necessary for
/// trading etc. against it while the loan is active.
contract OrderBookV4FlashLenderReentrant is OrderBookExternalRealTest {
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
    /// forge-config: default.fuzz.runs = 100
    function testReenterReadVaultBalances(uint256 vaultId, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV4.vaultBalance.selector, address(borrower), address(iToken0), vaultId),
            loanAmount
        );
    }

    /// Can reenter and check if an order exists from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterCheckOrderExists(bytes32 orderHash, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV4.orderExists.selector, orderHash), loanAmount
        );
    }

    /// Can reenter and deposit from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
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
            abi.encodeWithSelector(
                IOrderBookV4.deposit2.selector, address(iToken0), vaultId, depositAmount, new EvaluableV3[](0)
            ),
            loanAmount
        );
    }

    /// Can reenter and withdraw from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
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
            abi.encodeWithSelector(
                IOrderBookV4.withdraw2.selector, address(iToken0), vaultId, withdrawAmount, new TaskV1[](0)
            ),
            loanAmount
        );
    }

    /// Can reenter and add an order from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterAddOrder(uint256 loanAmount, OrderConfigV3 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV4.addOrder2.selector, config, new TaskV1[](0)), loanAmount
        );
    }

    /// Can reenter and remove an order from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterRemoveOrder(uint256 loanAmount, OrderV3 memory order) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        order.owner = address(borrower);
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV4.removeOrder2.selector, order, new TaskV1[](0)), loanAmount
        );
    }

    /// Can reenter and take orders.
    /// forge-config: default.fuzz.runs = 100
    function testReenterTakeOrder(uint256 loanAmount, OrderConfigV3 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        vm.recordLogs();
        iOrderbook.addOrder2(config, new TaskV1[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, OrderV3 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV3));

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](1);
        orders[0] = TakeOrderConfigV3(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV3 memory takeOrdersConfig =
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, "");

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV4.takeOrders2.selector, takeOrdersConfig), loanAmount
        );
    }

    /// Can reenter and clear orders.
    /// forge-config: default.fuzz.runs = 100
    function testReenterClear(
        uint256 loanAmount,
        address alice,
        address bob,
        OrderConfigV3 memory aliceConfig,
        OrderConfigV3 memory bobConfig
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(aliceConfig, iInterpreter, iStore);
        aliceConfig.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        LibTestAddOrder.conformConfig(bobConfig, iInterpreter, iStore);
        bobConfig.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        bobConfig.validInputs[0] = aliceConfig.validOutputs[0];
        bobConfig.validOutputs[0] = aliceConfig.validInputs[0];

        vm.recordLogs();
        vm.prank(alice);
        iOrderbook.addOrder2(aliceConfig, new TaskV1[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, OrderV3 memory aliceOrder) = abi.decode(entries[0].data, (address, bytes32, OrderV3));

        vm.recordLogs();
        vm.prank(bob);
        iOrderbook.addOrder2(bobConfig, new TaskV1[](0));
        entries = vm.getRecordedLogs();
        (,, OrderV3 memory bobOrder) = abi.decode(entries[0].data, (address, bytes32, OrderV3));

        ClearConfig memory clearConfig = ClearConfig(0, 0, 0, 0, 0, 0);

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(
                IOrderBookV4.clear2.selector,
                aliceOrder,
                bobOrder,
                clearConfig,
                new SignedContextV1[](0),
                new SignedContextV1[](0)
            ),
            loanAmount
        );
    }
}
