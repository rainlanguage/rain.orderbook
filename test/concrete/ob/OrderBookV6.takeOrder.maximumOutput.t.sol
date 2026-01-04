// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest, Vm} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    SignedContextV1,
    IOrderBookV6,
    OrderConfigV4,
    IOV2,
    EvaluableV4,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract OrderBookV6TakeOrderMaximumOutputTest is OrderBookV6ExternalRealTest {
    using LibDecimalFloat for Float;

    /// It should be possible to take an order with zero maximum output.
    function testTakeOrderMaximumOutputZero(OrderV4 memory order, SignedContextV1 memory signedContext) external {
        vm.assume(order.validInputs.length > 0);
        vm.assume(order.validOutputs.length > 0);
        order.validInputs[0].token = address(iToken0);
        order.validOutputs[0].token = address(iToken1);
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        orders[0] = TakeOrderConfigV4({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: signedContexts});
        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            orders: orders,
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIORatio: LibDecimalFloat.packLossless(1, 0),
            IOIsInput: false,
            data: ""
        });
        vm.expectRevert(IOrderBookV6.ZeroMaximumIO.selector);
        (Float totalTakerInput, Float totalTakerOutput) = iOrderbook.takeOrders4(config);
        (totalTakerInput, totalTakerOutput);
    }

    struct TestOrder {
        address owner;
        bytes orderString;
    }

    struct TestVault {
        address owner;
        address token;
        Float deposit;
        Float expect;
    }

    function checkTakeOrderMaximumOutput(
        TestOrder[] memory testOrders,
        TestVault[] memory testVaults,
        Float maximumTakerOutput,
        Float expectedTakerInput,
        Float expectedTakerOutput
    ) internal {
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));
        bytes32 vaultId = bytes32(uint256(0x01));

        OrderV4[] memory orders = new OrderV4[](testOrders.length);

        for (uint256 i = 0; i < testOrders.length; i++) {
            {
                OrderConfigV4 memory orderConfig;
                {
                    bytes memory bytecode = iParserV2.parse2(testOrders[i].orderString);
                    IOV2[] memory inputs = new IOV2[](1);
                    inputs[0] = IOV2({token: address(iToken0), vaultId: vaultId});
                    IOV2[] memory outputs = new IOV2[](1);
                    outputs[0] = IOV2({token: address(iToken1), vaultId: vaultId});
                    EvaluableV4 memory evaluable =
                        EvaluableV4({interpreter: iInterpreter, store: iStore, bytecode: bytecode});
                    orderConfig = OrderConfigV4({
                        evaluable: evaluable,
                        validInputs: inputs,
                        validOutputs: outputs,
                        nonce: bytes32(0),
                        secret: bytes32(0),
                        meta: ""
                    });
                }

                vm.prank(testOrders[i].owner);
                vm.recordLogs();
                iOrderbook.addOrder4(orderConfig, new TaskV2[](0));
                Vm.Log[] memory logs = vm.getRecordedLogs();
                assertEq(logs.length, 1);
                (,, OrderV4 memory order) = abi.decode(logs[0].data, (address, bytes32, OrderV4));
                orders[i] = order;
            }
        }

        for (uint256 i = 0; i < testVaults.length; i++) {
            if (testVaults[i].deposit.gt(Float.wrap(0))) {
                uint256 depositAmount18 = LibDecimalFloat.toFixedDecimalLossless(testVaults[i].deposit, 18);

                // Deposit the amount of tokens required to take the order.
                vm.mockCall(
                    address(iToken1),
                    abi.encodeWithSelector(
                        iToken1.transferFrom.selector, testVaults[i].owner, address(iOrderbook), depositAmount18
                    ),
                    abi.encode(true)
                );
                vm.expectCall(
                    address(iToken1),
                    abi.encodeWithSelector(
                        iToken1.transferFrom.selector, testVaults[i].owner, address(iOrderbook), depositAmount18
                    ),
                    1
                );
                Float balanceBefore = iOrderbook.vaultBalance2(testVaults[i].owner, testVaults[i].token, vaultId);
                vm.prank(testVaults[i].owner);
                iOrderbook.deposit4(testVaults[i].token, vaultId, testVaults[i].deposit, new TaskV2[](0));

                Float balanceAfter = iOrderbook.vaultBalance2(testVaults[i].owner, testVaults[i].token, vaultId);
                Float expectedBalance = testVaults[i].deposit.add(balanceBefore);

                assertTrue(balanceAfter.eq(expectedBalance), "deposit");
            }
        }

        TakeOrderConfigV4[] memory takeOrders = new TakeOrderConfigV4[](orders.length);
        for (uint256 i = 0; i < orders.length; i++) {
            takeOrders[i] = TakeOrderConfigV4({
                order: orders[i],
                inputIOIndex: 0,
                outputIOIndex: 0,
                signedContext: new SignedContextV1[](0)
            });
        }

        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            orders: takeOrders,
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: maximumTakerOutput,
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: false,
            data: ""
        });

        {
            (uint256 expectedTakerInput18,) = LibDecimalFloat.toFixedDecimalLossy(expectedTakerInput, 18);
            (uint256 expectedTakerOutput18, bool losslessOutput) =
                LibDecimalFloat.toFixedDecimalLossy(expectedTakerOutput, 18);
            if (!losslessOutput) {
                expectedTakerOutput18 += 1;
            }
            // Mock and expect the token transfers.
            vm.mockCall(
                address(iToken0),
                abi.encodeWithSelector(iToken0.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput18),
                abi.encode(true)
            );
            vm.expectCall(
                address(iToken0),
                abi.encodeWithSelector(iToken0.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput18),
                expectedTakerOutput18 > 0 ? 1 : 0
            );
            vm.mockCall(
                address(iToken1),
                abi.encodeWithSelector(iToken1.transfer.selector, bob, expectedTakerInput18),
                abi.encode(true)
            );
            vm.expectCall(
                address(iToken1),
                abi.encodeWithSelector(iToken1.transfer.selector, bob, expectedTakerInput18),
                expectedTakerInput18 > 0 ? 1 : 0
            );
        }
        {
            vm.prank(bob);
            (Float totalTakerInput, Float totalTakerOutput) = iOrderbook.takeOrders4(config);

            assertTrue(totalTakerInput.eq(expectedTakerInput), "taker input");
            assertTrue(totalTakerOutput.eq(expectedTakerOutput), "taker output");
        }

        for (uint256 i = 0; i < testVaults.length; i++) {
            Float finalBalance = iOrderbook.vaultBalance2(testVaults[i].owner, testVaults[i].token, vaultId);
            Float expectedFinalBalance = testVaults[i].expect;
            assertTrue(finalBalance.eq(expectedFinalBalance), "final balance");
        }
    }

    /// Add an order with unlimited maximum output and take it with a maximum
    /// taker output. Only the maximum taker output should be used.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumOutputSingleOrderUnlimitedMax(uint256 expectedTakerOutput18) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));

        expectedTakerOutput18 = bound(expectedTakerOutput18, 1, type(uint128).max);
        uint256 expectedTakerInput18 = expectedTakerOutput18 * 2;

        Float expectedTakerInput = LibDecimalFloat.fromFixedDecimalLosslessPacked(expectedTakerInput18, 18);
        Float expectedTakerOutput = LibDecimalFloat.fromFixedDecimalLosslessPacked(expectedTakerOutput18, 18);

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder({owner: owner, orderString: "_ _: max-positive-value() 0.5;:;"});

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] =
            TestVault({owner: owner, token: address(iToken1), deposit: expectedTakerInput, expect: Float.wrap(0)});
        testVaults[1] =
            TestVault({owner: owner, token: address(iToken0), deposit: Float.wrap(0), expect: expectedTakerOutput});

        checkTakeOrderMaximumOutput(testOrders, testVaults, expectedTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// Add an order with less than the maximum IO. Only the limit from the order
    /// should be used.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumOutputSingleOrderLimitedMax(uint256 maximumTakerOutput18) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        maximumTakerOutput18 = bound(maximumTakerOutput18, 1e18, uint256(int256(type(int224).max)));

        Float maximumTakerOutput = LibDecimalFloat.fromFixedDecimalLosslessPacked(maximumTakerOutput18, 18);

        Float expectedTakerInput = LibDecimalFloat.packLossless(5, -2);
        Float expectedTakerOutput = expectedTakerInput.mul(LibDecimalFloat.packLossless(2, 0));

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder({owner: owner, orderString: "_ _: 0.05 2;:;"});

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] =
            TestVault({owner: owner, token: address(iToken1), deposit: expectedTakerInput, expect: Float.wrap(0)});
        testVaults[1] =
            TestVault({owner: owner, token: address(iToken0), deposit: Float.wrap(0), expect: expectedTakerOutput});

        checkTakeOrderMaximumOutput(testOrders, testVaults, maximumTakerOutput, expectedTakerInput, expectedTakerOutput);
    }

    /// If the vault balance is less than both the maximum output and the order
    /// limit, only the vault balance should be used.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumOutputSingleOrderLimitedByVault(
        uint256 ownerDepositAmount18,
        uint256 maximumTakerInput18
    ) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));

        uint256 orderLimit18 = 1000;
        ownerDepositAmount18 = bound(ownerDepositAmount18, 0, orderLimit18 - 1);
        maximumTakerInput18 = bound(maximumTakerInput18, orderLimit18, uint256(int256(type(int224).max)));

        Float ownerDepositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(ownerDepositAmount18, 18);
        Float maximumTakerInput = LibDecimalFloat.fromFixedDecimalLosslessPacked(maximumTakerInput18, 18);

        Float orderIORatio = LibDecimalFloat.fromFixedDecimalLosslessPacked(2, 0);
        Float maximumTakerOutput = maximumTakerInput.mul(orderIORatio);

        Float expectedTakerInput = ownerDepositAmount;
        Float expectedTakerOutput = expectedTakerInput.mul(orderIORatio);

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder({owner: owner, orderString: "_ _: 1000e-18 2;:;"});

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] =
            TestVault({owner: owner, token: address(iToken1), deposit: ownerDepositAmount, expect: Float.wrap(0)});
        testVaults[1] =
            TestVault({owner: owner, token: address(iToken0), deposit: Float.wrap(0), expect: expectedTakerOutput});
        checkTakeOrderMaximumOutput(testOrders, testVaults, maximumTakerOutput, expectedTakerInput, expectedTakerOutput);
    }

    /// The deposit amount can be anything, the order taking should adjust
    /// accordingly, and leave any unspent deposited tokens in the vault.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumOutputSingleAnyDeposit(uint256 ownerDepositAmount18, uint256 maximumTakerOutput18)
        external
    {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit18 = 1000;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder({owner: owner, orderString: "_ _: 1000e-18 2;:;"});

        ownerDepositAmount18 = bound(ownerDepositAmount18, 0, uint256(int256(type(int224).max)));
        maximumTakerOutput18 = bound(maximumTakerOutput18, 1, uint256(int256(type(int224).max)));

        Float orderIO = LibDecimalFloat.fromFixedDecimalLosslessPacked(2, 0);
        Float orderLimit = LibDecimalFloat.fromFixedDecimalLosslessPacked(orderLimit18, 18);
        Float ownerDepositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(ownerDepositAmount18, 18);
        Float maximumTakerOutput = LibDecimalFloat.fromFixedDecimalLosslessPacked(maximumTakerOutput18, 18);
        Float maximumTakerInput = maximumTakerOutput.div(orderIO);

        Float expectedTakerInput = maximumTakerInput.min(ownerDepositAmount);
        expectedTakerInput = expectedTakerInput.min(orderLimit);

        Float expectedTakerOutput = expectedTakerInput.mul(orderIO);

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault({
            owner: owner,
            token: address(iToken1),
            deposit: ownerDepositAmount,
            expect: ownerDepositAmount.sub(expectedTakerInput)
        });
        testVaults[1] =
            TestVault({owner: owner, token: address(iToken0), deposit: Float.wrap(0), expect: expectedTakerOutput});
        checkTakeOrderMaximumOutput(testOrders, testVaults, maximumTakerOutput, expectedTakerInput, expectedTakerOutput);
    }

    /// The taker input can be sourced from multiple orders. Tests two orders
    /// that combined make up the maximum taker output. Both orders have the same
    /// owner for simplicity.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumOutputMultipleOrders(uint256 ownerDepositAmount18, uint256 maximumTakerOutput18)
        external
    {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit18 = 1500;

        TestOrder[] memory testOrders = new TestOrder[](2);
        testOrders[0] = TestOrder({owner: owner, orderString: "_ _: 1000e-18 2;:;"});
        testOrders[1] = TestOrder({owner: owner, orderString: "_ _: 500e-18 2;:;"});

        ownerDepositAmount18 = bound(ownerDepositAmount18, 0, uint256(int256(type(int224).max)));
        maximumTakerOutput18 = bound(maximumTakerOutput18, 1, uint256(int256(type(int224).max)));

        Float orderIO = LibDecimalFloat.fromFixedDecimalLosslessPacked(2, 0);
        Float orderLimit = LibDecimalFloat.fromFixedDecimalLosslessPacked(orderLimit18, 18);
        Float ownerDepositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(ownerDepositAmount18, 18);
        Float maximumTakerOutput = LibDecimalFloat.fromFixedDecimalLosslessPacked(maximumTakerOutput18, 18);
        Float maximumTakerInput = maximumTakerOutput.div(orderIO);

        Float expectedTakerInput = maximumTakerInput.min(ownerDepositAmount);
        expectedTakerInput = expectedTakerInput.min(orderLimit);
        Float expectedTakerOutput = expectedTakerInput.mul(orderIO);

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault({
            owner: owner,
            token: address(iToken1),
            deposit: ownerDepositAmount,
            expect: ownerDepositAmount.sub(expectedTakerInput)
        });
        testVaults[1] =
            TestVault({owner: owner, token: address(iToken0), deposit: Float.wrap(0), expect: expectedTakerOutput});

        checkTakeOrderMaximumOutput(testOrders, testVaults, maximumTakerOutput, expectedTakerInput, expectedTakerOutput);
    }

    /// The taker input can be sourced from multiple orders with different
    /// owners. Tests two orders that combined make up the maximum taker output.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderMaximumOutputMultipleOrdersDifferentOwners(
        uint256 ownerOneDepositAmount18,
        uint256 ownerTwoDepositAmount18,
        uint256 maximumTakerOutput18
    ) external {
        address ownerOne = address(uint160(uint256(keccak256("owner.one.rain.test"))));
        address ownerTwo = address(uint160(uint256(keccak256("owner.two.rain.test"))));

        ownerOneDepositAmount18 = bound(ownerOneDepositAmount18, 0, uint256(int256(type(int224).max)));
        ownerTwoDepositAmount18 =
            bound(ownerTwoDepositAmount18, 0, uint256(int256(type(int224).max)) - ownerOneDepositAmount18);
        maximumTakerOutput18 = bound(maximumTakerOutput18, 1, uint256(int256(type(int224).max)));

        TestOrder[] memory testOrders = new TestOrder[](2);
        testOrders[0] = TestOrder({owner: ownerOne, orderString: "_ _: 1000e-18 2;:;"});
        testOrders[1] = TestOrder({owner: ownerTwo, orderString: "_ _: 500e-18 2;:;"});

        // The first owner's deposit is fully used before the second owner's
        // deposit is used.
        TestVault[] memory testVaults = new TestVault[](2);

        Float ownerOneDepositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(ownerOneDepositAmount18, 18);
        Float ownerTwoDepositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(ownerTwoDepositAmount18, 18);
        Float ownerTwoOrderLimit = LibDecimalFloat.fromFixedDecimalLosslessPacked(500, 18);
        Float orderIO = LibDecimalFloat.fromFixedDecimalLosslessPacked(2, 0);
        Float maximumTakerOutput = LibDecimalFloat.fromFixedDecimalLosslessPacked(maximumTakerOutput18, 18);
        Float maximumTakerInput = maximumTakerOutput.div(orderIO);

        Float expectedTakerInput;
        Float ownerOneTakerInput;
        {
            Float ownerOneOrderLimit = LibDecimalFloat.fromFixedDecimalLosslessPacked(1000, 18);

            // Owner one can't pay more than either their deposit or their order
            // limit.
            Float ownerOneMaxPayment = ownerOneDepositAmount.min(ownerOneOrderLimit);
            // Taker input from owner one is either the maximum taker input or
            // what owner one can pay.
            ownerOneTakerInput = maximumTakerInput.min(ownerOneMaxPayment);
            expectedTakerInput = ownerOneTakerInput;
            testVaults[0] = TestVault({
                owner: ownerOne,
                token: address(iToken1),
                deposit: ownerOneDepositAmount,
                expect: ownerOneDepositAmount.sub(ownerOneTakerInput)
            });
        }

        {
            // Owner two can't pay more than either their deposit or their order
            // limit.
            Float ownerTwoMaxPayment = ownerTwoDepositAmount.min(ownerTwoOrderLimit);
            // Taker input from owner two is whatever is remaining after owner
            // one, up to what owner two can pay.
            Float remainingTakerInput = maximumTakerInput.sub(ownerOneTakerInput);
            Float ownerTwoTakerInput = remainingTakerInput.min(ownerTwoMaxPayment);
            expectedTakerInput = expectedTakerInput.add(ownerTwoTakerInput);
            testVaults[1] = TestVault({
                owner: ownerTwo,
                token: address(iToken1),
                deposit: ownerTwoDepositAmount,
                expect: ownerTwoDepositAmount.sub(ownerTwoTakerInput)
            });
        }
        Float expectedTakerOutput = expectedTakerInput.mul(orderIO);
        checkTakeOrderMaximumOutput(testOrders, testVaults, maximumTakerOutput, expectedTakerInput, expectedTakerOutput);
    }
}
