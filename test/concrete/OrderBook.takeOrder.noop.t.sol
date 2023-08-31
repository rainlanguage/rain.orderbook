// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "src/lib/LibOrder.sol";

import "test/util/abstract/OrderBookExternalRealTest.sol";
import {NoOrders} from "src/concrete/OrderBook.sol";

/// @title OrderBookTakeOrderNoopTest
/// @notice A test harness for testing the OrderBook takeOrder function. Focuses
/// on the no-op case.
contract OrderBookTakeOrderNoopTest is OrderBookExternalRealTest {
    using LibOrder for Order;

    /// Take orders makes no sense without any orders in the input array and the
    /// caller has full control over this so we error.
    function testTakeOrderNoopZeroOrders(address input, address output) external {
        TakeOrdersConfigV2 memory config =
            TakeOrdersConfigV2(output, input, 0, type(uint256).max, type(uint256).max, new TakeOrderConfig[](0), "");
        vm.expectRevert(NoOrders.selector);
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// If there is some order in the input array but it is not live we don't
    /// error as the caller may not have control over this, e.g. the order may
    /// have been removed by its owner. We don't want to revert the whole
    /// transaction in this case as there may be other orders in the input array
    /// in the general case.
    function testTakeOrderNoopNonLiveOrder(
        address input,
        address output,
        Order memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        SignedContextV1 memory signedContext
    ) external {
        // We don't bound the input or output indexes as we want to allow
        // malformed orders to be passed in, and still show that nothing happens.
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        TakeOrderConfig memory orderConfig = TakeOrderConfig(order, inputIOIndex, outputIOIndex, signedContexts);
        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        orders[0] = orderConfig;
        TakeOrdersConfigV2 memory config =
            TakeOrdersConfigV2(output, input, 0, type(uint256).max, type(uint256).max, orders, "");
        vm.expectEmit(address(iOrderbook));
        emit OrderNotFound(address(this), order.owner, order.hash());
        vm.recordLogs();
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        assertEq(totalTakerInput, 0);
        assertEq(totalTakerOutput, 0);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
    }

    /// Same as above but with two orders.
    function testTakeOrderNoopNonLiveOrderTwo(
        address input,
        address output,
        Order memory order1,
        Order memory order2,
        uint256 inputIOIndex1,
        uint256 outputIOIndex1,
        uint256 inputIOIndex2,
        uint256 outputIOIndex2,
        SignedContextV1 memory signedContext1,
        SignedContextV1 memory signedContext2
    ) external {
        TakeOrdersConfigV2 memory config;
        {
            TakeOrderConfig[] memory orders;
            {
                // We don't bound the input or output indexes as we want to allow
                // malformed orders to be passed in, and still show that nothing happens.
                SignedContextV1[] memory signedContexts1 = new SignedContextV1[](1);
                signedContexts1[0] = signedContext1;
                TakeOrderConfig memory orderConfig1 =
                    TakeOrderConfig(order1, inputIOIndex1, outputIOIndex1, signedContexts1);
                SignedContextV1[] memory signedContexts2 = new SignedContextV1[](1);
                signedContexts2[0] = signedContext2;
                TakeOrderConfig memory orderConfig2 =
                    TakeOrderConfig(order2, inputIOIndex2, outputIOIndex2, signedContexts2);
                orders = new TakeOrderConfig[](2);
                orders[0] = orderConfig1;
                orders[1] = orderConfig2;
            }

            config = TakeOrdersConfigV2(output, input, 0, type(uint256).max, type(uint256).max, orders, "");
        }

        vm.recordLogs();
        {
            (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
            assertEq(totalTakerInput, 0);
            assertEq(totalTakerOutput, 0);
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
