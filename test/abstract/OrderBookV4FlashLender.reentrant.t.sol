// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {
    IOrderBookV5,
    OrderConfigV4,
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV4,
    ClearConfigV2,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {EvaluableV4, SignedContextV1} from "rain.interpreter.interface/interface/unstable/IInterpreterCallerV4.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

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
            abi.encodeWithSelector(IOrderBookV5.vaultBalance2.selector, address(borrower), address(iToken0), vaultId),
            loanAmount
        );
    }

    /// Can reenter and check if an order exists from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterCheckOrderExists(bytes32 orderHash, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV5.orderExists.selector, orderHash), loanAmount
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
                IOrderBookV5.deposit3.selector, address(iToken0), vaultId, depositAmount, new EvaluableV4[](0)
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
                IOrderBookV5.withdraw3.selector, address(iToken0), vaultId, withdrawAmount, new TaskV2[](0)
            ),
            loanAmount
        );
    }

    /// Can reenter and add an order from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterAddOrder(uint256 loanAmount, OrderConfigV4 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV5.addOrder3.selector, config, new TaskV2[](0)), loanAmount
        );
    }

    /// Can reenter and remove an order from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterRemoveOrder(uint256 loanAmount, OrderV4 memory order) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        order.owner = address(borrower);
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV5.removeOrder3.selector, order, new TaskV2[](0)), loanAmount
        );
    }

    /// Can reenter and take orders.
    /// forge-config: default.fuzz.runs = 100
    function testReenterTakeOrder(uint256 loanAmount, OrderConfigV4 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        vm.recordLogs();
        iOrderbook.addOrder3(config, new TaskV2[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, OrderV4 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV4));

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV4 memory takeOrdersConfig =
            TakeOrdersConfigV4(LibDecimalFloat.packLossless(0, 0), LibDecimalFloat.packLossless(type(int256).max, 0), LibDecimalFloat.packLossless(type(int256).max, 0), orders, "");

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV5.takeOrders3.selector, takeOrdersConfig), loanAmount
        );
    }

    /// Can reenter and clear orders.
    /// forge-config: default.fuzz.runs = 100
    function testReenterClear(
        uint256 loanAmount,
        address alice,
        address bob,
        OrderConfigV4 memory aliceConfig,
        OrderConfigV4 memory bobConfig
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(aliceConfig, iInterpreter, iStore);
        aliceConfig.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        LibTestAddOrder.conformConfig(bobConfig, iInterpreter, iStore);
        bobConfig.evaluable.bytecode = iParserV2.parse2("_ _:max-value() 1;:;");

        bobConfig.validInputs[0] = aliceConfig.validOutputs[0];
        bobConfig.validOutputs[0] = aliceConfig.validInputs[0];

        vm.mockCall(bobConfig.validInputs[0].token, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));
        vm.mockCall(bobConfig.validOutputs[0].token, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));

        vm.recordLogs();
        vm.prank(alice);
        iOrderbook.addOrder3(aliceConfig, new TaskV2[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, OrderV4 memory aliceOrder) = abi.decode(entries[0].data, (address, bytes32, OrderV4));

        vm.recordLogs();
        vm.prank(bob);
        iOrderbook.addOrder3(bobConfig, new TaskV2[](0));
        entries = vm.getRecordedLogs();
        (,, OrderV4 memory bobOrder) = abi.decode(entries[0].data, (address, bytes32, OrderV4));

        ClearConfigV2 memory clearConfig = ClearConfigV2(0, 0, 0, 0, 0, 0);

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(
                IOrderBookV5.clear3.selector,
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
