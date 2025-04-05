// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookExternalRealTest, Vm} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV4,
    ZeroMaximumInput,
    IOV2,
    EvaluableV4,
    OrderConfigV4,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/deprecated/IInterpreterCallerV2.sol";

contract OrderBookTakeOrderMaximumInputTest is OrderBookExternalRealTest {
    /// If there is some live order(s) but the maxTakerInput is zero we error as
    /// the caller has full control over this, and it would cause none of the
    /// orders to be taken.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderNoopZeroMaxTakerInput(OrderV4 memory order, SignedContextV1 memory signedContext) external {
        vm.assume(order.validInputs.length > 0);
        vm.assume(order.validOutputs.length > 0);
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        orders[0] = TakeOrderConfigV4(order, 0, 0, signedContexts);
        TakeOrdersConfigV4 memory config = TakeOrdersConfigV4(0, 0, type(uint256).max, orders, "");
        vm.expectRevert(ZeroMaximumInput.selector);
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders2(config);
        (totalTakerInput, totalTakerOutput);
    }

    struct TestOrder {
        address owner;
        bytes orderString;
    }

    struct TestVault {
        address owner;
        address token;
        uint256 deposit;
        uint256 expect;
    }

    function checkTakeOrderMaximumInput(
        TestOrder[] memory testOrders,
        TestVault[] memory testVaults,
        uint256 maximumTakerInput,
        uint256 expectedTakerInput,
        uint256 expectedTakerOutput
    ) internal {
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));
        uint256 vaultId = 0;

        OrderV4[] memory orders = new OrderV4[](testOrders.length);

        for (uint256 i = 0; i < testOrders.length; i++) {
            {
                OrderConfigV4 memory orderConfig;
                {
                    bytes memory bytecode = iParserV2.parse2(testOrders[i].orderString);
                    IOV2[] memory inputs = new IOV2[](1);
                    inputs[0] = IOV2(address(iToken0), 18, vaultId);
                    IOV2[] memory outputs = new IOV2[](1);
                    outputs[0] = IOV2(address(iToken1), 18, vaultId);
                    EvaluableV4 memory evaluable = EvaluableV4(iInterpreter, iStore, bytecode);
                    orderConfig = OrderConfigV4(evaluable, inputs, outputs, bytes32(0), bytes32(0), "");
                }

                vm.prank(testOrders[i].owner);
                vm.recordLogs();
                iOrderbook.addOrder3(orderConfig, new TaskV2[](0));
                Vm.Log[] memory entries = vm.getRecordedLogs();
                assertEq(entries.length, 1);
                (,, OrderV4 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV4));
                orders[i] = order;
            }
        }

        for (uint256 i = 0; i < testVaults.length; i++) {
            if (testVaults[i].deposit > 0) {
                // Deposit the amount of tokens required to take the order.
                vm.mockCall(
                    address(iToken1),
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, testVaults[i].owner, address(iOrderbook), testVaults[i].deposit
                    ),
                    abi.encode(true)
                );
                vm.expectCall(
                    address(iToken1),
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, testVaults[i].owner, address(iOrderbook), testVaults[i].deposit
                    ),
                    1
                );
                uint256 balanceBefore = iOrderbook.vaultBalance(testVaults[i].owner, testVaults[i].token, vaultId);
                vm.prank(testVaults[i].owner);
                iOrderbook.deposit3(testVaults[i].token, vaultId, testVaults[i].deposit, new TaskV2[](0));
                assertEq(
                    iOrderbook.vaultBalance(testVaults[i].owner, testVaults[i].token, vaultId),
                    balanceBefore + testVaults[i].deposit,
                    "vaultBalance before"
                );
            }
        }

        TakeOrderConfigV4[] memory takeOrders = new TakeOrderConfigV4[](orders.length);
        for (uint256 i = 0; i < orders.length; i++) {
            takeOrders[i] = TakeOrderConfigV4(orders[i], 0, 0, new SignedContextV1[](0));
        }
        TakeOrdersConfigV4 memory config = TakeOrdersConfigV4(0, maximumTakerInput, type(uint256).max, takeOrders, "");

        // Mock and expect the token transfers.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transfer.selector, bob, expectedTakerInput),
            abi.encode(true)
        );
        vm.expectCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transfer.selector, bob, expectedTakerInput),
            expectedTakerInput > 0 ? 1 : 0
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput),
            abi.encode(true)
        );
        vm.expectCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput),
            expectedTakerOutput > 0 ? 1 : 0
        );

        vm.prank(bob);
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders2(config);
        assertEq(totalTakerInput, expectedTakerInput, "totalTakerInput");
        assertEq(totalTakerOutput, expectedTakerOutput, "totalTakerOutput");

        for (uint256 i = 0; i < testVaults.length; i++) {
            assertEq(
                iOrderbook.vaultBalance(testVaults[i].owner, testVaults[i].token, vaultId),
                testVaults[i].expect,
                "vaultBalance"
            );
        }
    }

    /// Add an order with unlimited maximum output and take it with a maximum
    /// input. Only the maximum input should be taken.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumInputSingleOrderUnlimitedMax(uint256 expectedTakerInput) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));

        expectedTakerInput = bound(expectedTakerInput, 1, type(uint128).max);
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:max-value() 2;:;");

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), expectedTakerInput, 0);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, expectedTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// Add an order with less than the maximum output. Only the limit from the
    /// order should be taken.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumInputSingleOrderLessThanMaximumOutput(uint256 maximumTakerInput) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        maximumTakerInput = bound(maximumTakerInput, 1000, type(uint256).max);
        uint256 expectedTakerInput = 1000;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:1e-15 2;:;");

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), expectedTakerInput, 0);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// If the vault balance is less than both the maximum input and the order
    /// limit, the vault balance should be taken.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumInputSingleOrderLessThanMaximumInput(
        uint256 ownerDepositAmount,
        uint256 maximumTakerInput
    ) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit = 1000;
        ownerDepositAmount = bound(ownerDepositAmount, 0, orderLimit - 1);
        maximumTakerInput = bound(maximumTakerInput, 1000, type(uint256).max);
        uint256 expectedTakerInput = ownerDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:1e-15 2;:;");

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), ownerDepositAmount, 0);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// The deposit amount can be anything actually, the order taking should
    /// adjust accordingly, and leave any unspent deposited tokens in the vault.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumInputSingleAnyDeposit(uint256 ownerDepositAmount, uint256 maximumTakerInput)
        external
    {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit = 1000;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:1e-15 2;:;");

        maximumTakerInput = bound(maximumTakerInput, 1, type(uint256).max);
        // The expected input is the minimum of the maximum input and the order
        // limit.
        uint256 expectedTakerInput = maximumTakerInput < orderLimit ? maximumTakerInput : orderLimit;

        expectedTakerInput = expectedTakerInput < ownerDepositAmount ? expectedTakerInput : ownerDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), ownerDepositAmount, ownerDepositAmount - expectedTakerInput);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// The taker input can be sourced from multiple orders. Tests two orders
    /// that combined make up the maximum taker input. Both orders have the
    /// same owner.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumInputMultipleOrders(uint256 ownerDepositAmount, uint256 maximumTakerInput) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit = 1500;

        TestOrder[] memory testOrders = new TestOrder[](2);
        testOrders[0] = TestOrder(owner, "_ _:1e-15 2;:;");
        testOrders[1] = TestOrder(owner, "_ _:5e-16 2;:;");

        maximumTakerInput = bound(maximumTakerInput, 1, type(uint256).max);
        // The expected input is the minimum of the maximum input and the order
        // limit.
        uint256 expectedTakerInput = maximumTakerInput < orderLimit ? maximumTakerInput : orderLimit;

        expectedTakerInput = expectedTakerInput < ownerDepositAmount ? expectedTakerInput : ownerDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), ownerDepositAmount, ownerDepositAmount - expectedTakerInput);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// The taker input can be source from multiple orders with multiple owners.
    /// Tests two orders that combined make up the maximum taker input. Both
    /// orders have different owners.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumInputMultipleOrdersMultipleOwners(
        uint256 ownerOneDepositAmount,
        uint256 ownerTwoDepositAmount,
        uint256 maximumTakerInput
    ) external {
        // Avoid information free overflow.
        ownerTwoDepositAmount = bound(ownerTwoDepositAmount, 0, type(uint256).max - ownerOneDepositAmount);

        address ownerOne = address(uint160(uint256(keccak256("ownerOne.rain.test"))));
        address ownerTwo = address(uint160(uint256(keccak256("ownerTwo.rain.test"))));

        TestOrder[] memory testOrders = new TestOrder[](2);
        testOrders[0] = TestOrder(ownerOne, "_ _:1e-15 2;:;");
        testOrders[1] = TestOrder(ownerTwo, "_ _:5e-16 2;:;");

        maximumTakerInput = bound(maximumTakerInput, 1, type(uint256).max);

        // The first owner's deposit is fully used before the second owner's
        // deposit is used.
        TestVault[] memory testVaults = new TestVault[](2);

        uint256 expectedTakerInput;
        uint256 ownerOneTakerInput;
        {
            // Owner one can't pay more than either their deposit or 1000 set in
            // the order.
            uint256 ownerOneMaxPayment = ownerOneDepositAmount < 1000 ? ownerOneDepositAmount : 1000;
            // taker input from owner one is either the maximum taker input if
            // it is less than the max owner one payment, or the max owner one
            // payment.
            ownerOneTakerInput = maximumTakerInput < ownerOneMaxPayment ? maximumTakerInput : ownerOneMaxPayment;
            testVaults[0] =
                TestVault(ownerOne, address(iToken1), ownerOneDepositAmount, ownerOneDepositAmount - ownerOneTakerInput);
        }

        {
            // Owner two can't pay more than either their deposit or 500 set in
            // the order.
            uint256 ownerTwoMaxPayment = ownerTwoDepositAmount < 500 ? ownerTwoDepositAmount : 500;
            // Taker input from owner two is either whatever is remaining after
            // owner one's payment, or the max owner two payment.
            uint256 ownerTwoTakerInput =
                ownerOneTakerInput < maximumTakerInput ? maximumTakerInput - ownerOneTakerInput : 0;
            ownerTwoTakerInput = ownerTwoTakerInput < ownerTwoMaxPayment ? ownerTwoTakerInput : ownerTwoMaxPayment;
            testVaults[1] =
                TestVault(ownerTwo, address(iToken1), ownerTwoDepositAmount, ownerTwoDepositAmount - ownerTwoTakerInput);
            expectedTakerInput = ownerOneTakerInput + ownerTwoTakerInput;
        }
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }
}
