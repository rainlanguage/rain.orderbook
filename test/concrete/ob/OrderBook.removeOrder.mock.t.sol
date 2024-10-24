// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {OrderConfigV3, OrderV3, EvaluableV3, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookRemoveOrderMockTest
/// @notice A contract to test the OrderBook removeOrder function.
contract OrderBookRemoveOrderMockTest is OrderBookExternalMockTest {
    /// An order MUST ONLY be removable by its owner.
    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOnlyOwner(address alice, address bob, OrderConfigV3 memory config, bytes memory expression)
        external
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        (OrderV3 memory expectedOrder, bytes32 expectedOrderHash) = LibTestAddOrder.expectedOrder(alice, config);

        // It will revert even if the order has not been added yet.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder2(expectedOrder, new TaskV1[](0));

        // And will revert after the order is added.
        (OrderV3 memory order, bytes32 orderHash) = addOrderWithChecks(alice, config, expression);
        assertEq(orderHash, expectedOrderHash);

        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder2(order, new TaskV1[](0));

        // Alice can remove the order.
        removeOrderWithChecks(alice, order);

        // It will revert even after the order has been removed.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder2(order, new TaskV1[](0));
    }

    /// The same order can be added and removed multiple times.
    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderAddRemoveMulti(address alice, OrderConfigV3 memory config, bytes memory expression)
        external
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        OrderV3 memory order;
        bytes32 orderHashA;
        bytes32 orderHashB;
        // Each iteration is quite slow so 3 is about as much as we want to do.
        for (uint256 i = 0; i < 3; i++) {
            (order, orderHashB) = addOrderWithChecks(alice, config, expression);
            removeOrderWithChecks(alice, order);
            if (i > 0) {
                assertEq(orderHashA, orderHashB);
            }
            orderHashA = orderHashB;
        }
    }

    /// An order MUST NOT change state if it does not exist.
    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderDoesNotExist(address alice, OrderConfigV3 memory config, bytes memory) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        (OrderV3 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(alice, config);
        assertFalse(iOrderbook.orderExists(orderHash));
        vm.record();
        vm.recordLogs();
        vm.prank(alice);
        assertFalse(iOrderbook.removeOrder2(order, new TaskV1[](0)));
        assertEq(vm.getRecordedLogs().length, 0);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
    }

    /// Can add and remove different orders.
    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderDifferent(
        address alice,
        OrderConfigV3 memory configOne,
        bytes memory expressionOne,
        OrderConfigV3 memory configTwo,
        bytes memory expressionTwo
    ) external {
        LibTestAddOrder.conformConfig(configOne, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configTwo, iInterpreter, iStore);

        (OrderV3 memory expectedOrderOne, bytes32 expectedOrderHashOne) =
            LibTestAddOrder.expectedOrder(alice, configOne);
        (OrderV3 memory expectedOrderTwo, bytes32 expectedOrderHashTwo) =
            LibTestAddOrder.expectedOrder(alice, configTwo);
        (expectedOrderOne);
        (expectedOrderTwo);
        vm.assume(expectedOrderHashOne != expectedOrderHashTwo);

        (OrderV3 memory orderOne, bytes32 orderHashOne) = addOrderWithChecks(alice, configOne, expressionOne);
        (OrderV3 memory orderTwo, bytes32 orderHashTwo) = addOrderWithChecks(alice, configTwo, expressionTwo);
        assertEq(orderHashOne, expectedOrderHashOne);
        assertEq(orderHashTwo, expectedOrderHashTwo);
        removeOrderWithChecks(alice, orderOne);
        removeOrderWithChecks(alice, orderTwo);
    }

    /// Different owners can add and remove the same order.
    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderDifferentOwners(
        address alice,
        address bob,
        OrderConfigV3 memory config,
        bytes memory expression
    ) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);
        (OrderV3 memory orderAlice, bytes32 orderHashAlice) = addOrderWithChecks(alice, config, expression);
        (OrderV3 memory orderBob, bytes32 orderHashBob) = addOrderWithChecks(bob, config, expression);
        assertTrue(orderHashAlice != orderHashBob);

        // Owners can't interfere with each other.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, alice, bob));
        vm.prank(alice);
        iOrderbook.removeOrder2(orderBob, new TaskV1[](0));

        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder2(orderAlice, new TaskV1[](0));

        removeOrderWithChecks(alice, orderAlice);
        removeOrderWithChecks(bob, orderBob);
    }

    /// Different owners can add and remove different orders.
    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderDifferentOwnersDifferent(
        address alice,
        address bob,
        OrderConfigV3 memory configOne,
        bytes memory expressionOne,
        OrderConfigV3 memory configTwo,
        bytes memory expressionTwo
    ) external {
        {
            LibTestAddOrder.conformConfig(configOne, iInterpreter, iStore);
            LibTestAddOrder.conformConfig(configTwo, iInterpreter, iStore);
            vm.assume(alice != bob);

            // Ensure the configs are different.
            (OrderV3 memory expectedOrderOne, bytes32 expectedOrderHashOne) =
                LibTestAddOrder.expectedOrder(address(0), configOne);
            (OrderV3 memory expectedOrderTwo, bytes32 expectedOrderHashTwo) =
                LibTestAddOrder.expectedOrder(address(0), configTwo);
            (expectedOrderOne);
            (expectedOrderTwo);
            vm.assume(expectedOrderHashOne != expectedOrderHashTwo);
        }

        OrderV3 memory orderAliceOne;
        OrderV3 memory orderBobOne;
        OrderV3 memory orderAliceTwo;
        OrderV3 memory orderBobTwo;
        {
            bytes32 orderHashAliceOne;
            bytes32 orderHashBobOne;
            bytes32 orderHashAliceTwo;
            bytes32 orderHashBobTwo;

            (orderAliceOne, orderHashAliceOne) = addOrderWithChecks(alice, configOne, expressionOne);
            (orderBobOne, orderHashBobOne) = addOrderWithChecks(bob, configOne, expressionOne);
            (orderAliceTwo, orderHashAliceTwo) = addOrderWithChecks(alice, configTwo, expressionTwo);
            (orderBobTwo, orderHashBobTwo) = addOrderWithChecks(bob, configTwo, expressionTwo);
            assertTrue(orderHashAliceOne != orderHashAliceTwo);
            assertTrue(orderHashAliceOne != orderHashBobOne);
            assertTrue(orderHashAliceOne != orderHashBobTwo);
            assertTrue(orderHashAliceTwo != orderHashBobOne);
            assertTrue(orderHashAliceTwo != orderHashBobTwo);
            assertTrue(orderHashBobOne != orderHashBobTwo);
        }

        // Owners can't interfere with each other.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, alice, bob));
        vm.prank(alice);
        iOrderbook.removeOrder2(orderBobOne, new TaskV1[](0));
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, alice, bob));
        vm.prank(alice);
        iOrderbook.removeOrder2(orderBobTwo, new TaskV1[](0));

        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder2(orderAliceOne, new TaskV1[](0));
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder2(orderAliceTwo, new TaskV1[](0));

        removeOrderWithChecks(alice, orderAliceOne);
        removeOrderWithChecks(bob, orderBobOne);
        removeOrderWithChecks(alice, orderAliceTwo);
        removeOrderWithChecks(bob, orderBobTwo);
    }
}
