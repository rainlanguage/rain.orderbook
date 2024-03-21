// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {OrderConfigV2, OrderV2} from "rain.orderbook.interface/interface/unstable/IOrderBookV3.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookRemoveOrderMockTest
/// @notice A contract to test the OrderBook removeOrder function.
contract OrderBookRemoveOrderMockTest is OrderBookExternalMockTest {
    /// An order MUST ONLY be removable by its owner.
    function testRemoveOrderOnlyOwner(address alice, address bob, OrderConfigV2 memory config, address expression)
        external
    {
        LibTestAddOrder.conformConfig(config, iDeployer);
        vm.assume(alice != bob);

        (OrderV2 memory expectedOrder, bytes32 expectedOrderHash) =
            LibTestAddOrder.expectedOrder(alice, config, iInterpreter, iStore, expression);

        // It will revert even if the order has not been added yet.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder(expectedOrder);

        // And will revert after the order is added.
        (OrderV2 memory order, bytes32 orderHash) = addOrderWithChecks(alice, config, expression);
        assertEq(orderHash, expectedOrderHash);

        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder(order);

        // Alice can remove the order.
        removeOrderWithChecks(alice, order);

        // It will revert even after the order has been removed.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder(order);
    }

    /// The same order can be added and removed multiple times.
    function testRemoveOrderAddRemoveMulti(address alice, OrderConfigV2 memory config, address expression) external {
        LibTestAddOrder.conformConfig(config, iDeployer);

        OrderV2 memory order;
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
    function testRemoveOrderDoesNotExist(address alice, OrderConfigV2 memory config, address expression) external {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (OrderV2 memory order, bytes32 orderHash) =
            LibTestAddOrder.expectedOrder(alice, config, iInterpreter, iStore, expression);
        assertFalse(iOrderbook.orderExists(orderHash));
        vm.record();
        vm.recordLogs();
        vm.prank(alice);
        assertFalse(iOrderbook.removeOrder(order));
        assertEq(vm.getRecordedLogs().length, 0);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
    }

    /// Can add and remove different orders.
    function testRemoveOrderDifferent(
        address alice,
        OrderConfigV2 memory configOne,
        address expressionOne,
        OrderConfigV2 memory configTwo,
        address expressionTwo
    ) external {
        LibTestAddOrder.conformConfig(configOne, iDeployer);
        LibTestAddOrder.conformConfig(configTwo, iDeployer);

        (OrderV2 memory expectedOrderOne, bytes32 expectedOrderHashOne) =
            LibTestAddOrder.expectedOrder(alice, configOne, iInterpreter, iStore, expressionOne);
        (OrderV2 memory expectedOrderTwo, bytes32 expectedOrderHashTwo) =
            LibTestAddOrder.expectedOrder(alice, configTwo, iInterpreter, iStore, expressionTwo);
        (expectedOrderOne);
        (expectedOrderTwo);
        vm.assume(expectedOrderHashOne != expectedOrderHashTwo);

        (OrderV2 memory orderOne, bytes32 orderHashOne) = addOrderWithChecks(alice, configOne, expressionOne);
        (OrderV2 memory orderTwo, bytes32 orderHashTwo) = addOrderWithChecks(alice, configTwo, expressionTwo);
        assertEq(orderHashOne, expectedOrderHashOne);
        assertEq(orderHashTwo, expectedOrderHashTwo);
        removeOrderWithChecks(alice, orderOne);
        removeOrderWithChecks(alice, orderTwo);
    }

    /// Different owners can add and remove the same order.
    function testRemoveOrderDifferentOwners(address alice, address bob, OrderConfigV2 memory config, address expression)
        external
    {
        LibTestAddOrder.conformConfig(config, iDeployer);
        vm.assume(alice != bob);
        (OrderV2 memory orderAlice, bytes32 orderHashAlice) = addOrderWithChecks(alice, config, expression);
        (OrderV2 memory orderBob, bytes32 orderHashBob) = addOrderWithChecks(bob, config, expression);
        assertTrue(orderHashAlice != orderHashBob);

        // Owners can't interfere with each other.
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, alice, bob));
        vm.prank(alice);
        iOrderbook.removeOrder(orderBob);

        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder(orderAlice);

        removeOrderWithChecks(alice, orderAlice);
        removeOrderWithChecks(bob, orderBob);
    }

    /// Different owners can add and remove different orders.
    function testRemoveOrderDifferentOwnersDifferent(
        address alice,
        address bob,
        OrderConfigV2 memory configOne,
        address expressionOne,
        OrderConfigV2 memory configTwo,
        address expressionTwo
    ) external {
        {
            LibTestAddOrder.conformConfig(configOne, iDeployer);
            LibTestAddOrder.conformConfig(configTwo, iDeployer);
            vm.assume(alice != bob);

            // Ensure the configs are different.
            (OrderV2 memory expectedOrderOne, bytes32 expectedOrderHashOne) =
                LibTestAddOrder.expectedOrder(address(0), configOne, iInterpreter, iStore, expressionOne);
            (OrderV2 memory expectedOrderTwo, bytes32 expectedOrderHashTwo) =
                LibTestAddOrder.expectedOrder(address(0), configTwo, iInterpreter, iStore, expressionTwo);
            (expectedOrderOne);
            (expectedOrderTwo);
            vm.assume(expectedOrderHashOne != expectedOrderHashTwo);
        }

        OrderV2 memory orderAliceOne;
        OrderV2 memory orderBobOne;
        OrderV2 memory orderAliceTwo;
        OrderV2 memory orderBobTwo;
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
        iOrderbook.removeOrder(orderBobOne);
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, alice, bob));
        vm.prank(alice);
        iOrderbook.removeOrder(orderBobTwo);

        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder(orderAliceOne);
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        vm.prank(bob);
        iOrderbook.removeOrder(orderAliceTwo);

        removeOrderWithChecks(alice, orderAliceOne);
        removeOrderWithChecks(bob, orderBobOne);
        removeOrderWithChecks(alice, orderAliceTwo);
        removeOrderWithChecks(bob, orderBobTwo);
    }
}
