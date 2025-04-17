// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Test.sol";
import {LibOrder} from "src/lib/LibOrder.sol";

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {NoOrders} from "src/concrete/ob/OrderBook.sol";
import {
    OrderV4,
    TakeOrdersConfigV4,
    TakeOrderConfigV4,
    SignedContextV1,
    EvaluableV4
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

/// @title OrderBookTakeOrderNoopTest
/// @notice A test harness for testing the OrderBook takeOrder function. Focuses
/// on the no-op case.
contract OrderBookTakeOrderNoopTest is OrderBookExternalRealTest {
    using LibOrder for OrderV4;
    using LibDecimalFloat for Float;

    /// Take orders makes no sense without any orders in the input array and the
    /// caller has full control over this so we error.
    function testTakeOrderNoopZeroOrders() external {
        TakeOrdersConfigV4 memory config = TakeOrdersConfigV4(
            Float(0, 0), Float(type(int256).max, 0), Float(type(int256).max, 0), new TakeOrderConfigV4[](0), ""
        );
        vm.expectRevert(NoOrders.selector);
        (Float memory totalTakerInput, Float memory totalTakerOutput) = iOrderbook.takeOrders3(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// If there is some order in the input array but it is not live we don't
    /// error as the caller may not have control over this, e.g. the order may
    /// have been removed by its owner. We don't want to revert the whole
    /// transaction in this case as there may be other orders in the input array
    /// in the general case.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderNoopNonLiveOrderOne(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        SignedContextV1 memory signedContext
    ) external {
        vm.assume(order.validInputs.length > 0);
        inputIOIndex = bound(inputIOIndex, 0, order.validInputs.length - 1);
        vm.assume(order.validOutputs.length > 0);
        outputIOIndex = bound(outputIOIndex, 0, order.validOutputs.length - 1);

        vm.assume(order.validInputs[inputIOIndex].token != order.validOutputs[outputIOIndex].token);

        // We don't bound the input or output indexes as we want to allow
        // malformed orders to be passed in, and still show that nothing happens.
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        TakeOrderConfigV4 memory orderConfig = TakeOrderConfigV4(order, inputIOIndex, outputIOIndex, signedContexts);
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = orderConfig;
        TakeOrdersConfigV4 memory config =
            TakeOrdersConfigV4(Float(0, 0), Float(type(int256).max, 0), Float(type(int256).max, 0), orders, "");
        vm.expectEmit(address(iOrderbook));
        emit OrderNotFound(address(this), order.owner, order.hash());
        vm.recordLogs();
        (Float memory totalTakerInput, Float memory totalTakerOutput) = iOrderbook.takeOrders3(config);
        assertTrue(totalTakerInput.eq(Float(0, 0)));
        assertTrue(totalTakerOutput.eq(Float(0, 0)));
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
    }

    /// Same as above but with two orders.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderNoopNonLiveOrderTwo(
        OrderV4 memory order1,
        OrderV4 memory order2,
        uint256 inputIOIndex1,
        uint256 outputIOIndex1,
        uint256 inputIOIndex2,
        uint256 outputIOIndex2,
        SignedContextV1 memory signedContext1,
        SignedContextV1 memory signedContext2
    ) external {
        vm.assume(order1.validInputs.length > 0);
        inputIOIndex1 = bound(inputIOIndex1, 0, order1.validInputs.length - 1);
        vm.assume(order1.validOutputs.length > 0);
        outputIOIndex1 = bound(outputIOIndex1, 0, order1.validOutputs.length - 1);
        vm.assume(order2.validInputs.length > 0);
        inputIOIndex2 = bound(inputIOIndex2, 0, order2.validInputs.length - 1);
        vm.assume(order2.validOutputs.length > 0);
        outputIOIndex2 = bound(outputIOIndex2, 0, order2.validOutputs.length - 1);

        vm.assume(order1.validInputs[inputIOIndex1].token != order1.validOutputs[outputIOIndex1].token);
        vm.assume(order2.validInputs[inputIOIndex2].token != order2.validOutputs[outputIOIndex2].token);

        // The inputs and outputs need to match or we will trigger the token
        // mismatch error.
        order1.validInputs[inputIOIndex1].token = order2.validInputs[inputIOIndex2].token;
        order1.validOutputs[outputIOIndex1].token = order2.validOutputs[outputIOIndex2].token;

        vm.mockCall(address(order1.validInputs[inputIOIndex1].token), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));
        vm.mockCall(address(order1.validOutputs[outputIOIndex1].token), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));

        TakeOrdersConfigV4 memory config;
        {
            TakeOrderConfigV4[] memory orders;
            {
                // We don't bound the input or output indexes as we want to allow
                // malformed orders to be passed in, and still show that nothing happens.
                SignedContextV1[] memory signedContexts1 = new SignedContextV1[](1);
                signedContexts1[0] = signedContext1;
                TakeOrderConfigV4 memory orderConfig1 =
                    TakeOrderConfigV4(order1, inputIOIndex1, outputIOIndex1, signedContexts1);
                SignedContextV1[] memory signedContexts2 = new SignedContextV1[](1);
                signedContexts2[0] = signedContext2;
                TakeOrderConfigV4 memory orderConfig2 =
                    TakeOrderConfigV4(order2, inputIOIndex2, outputIOIndex2, signedContexts2);
                orders = new TakeOrderConfigV4[](2);
                orders[0] = orderConfig1;
                orders[1] = orderConfig2;
            }

            config = TakeOrdersConfigV4(Float(0, 0), Float(type(int256).max, 0), Float(type(int256).max, 0), orders, "");
        }

        vm.recordLogs();
        {
            (Float memory totalTakerInput, Float memory totalTakerOutput) = iOrderbook.takeOrders3(config);
            assertTrue(totalTakerInput.eq(Float(0, 0)));
            assertTrue(totalTakerOutput.eq(Float(0, 0)));
        }
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 2);

        {
            assertEq(logs[0].topics.length, 1);
            assertEq(logs[0].topics[0], bytes32(uint256(keccak256("OrderNotFound(address,address,bytes32)"))));
            (address sender1, address owner1, bytes32 orderHash1) =
                abi.decode(logs[0].data, (address, address, bytes32));
            assertEq(sender1, address(this));
            assertEq(owner1, order1.owner);
            assertEq(orderHash1, order1.hash());
        }

        {
            assertEq(logs[1].topics.length, 1);
            assertEq(logs[1].topics[0], bytes32(uint256(keccak256("OrderNotFound(address,address,bytes32)"))));
            (address sender2, address owner2, bytes32 orderHash2) =
                abi.decode(logs[1].data, (address, address, bytes32));
            assertEq(sender2, address(this));
            assertEq(owner2, order2.owner);
            assertEq(orderHash2, order2.hash());
        }
    }
}
