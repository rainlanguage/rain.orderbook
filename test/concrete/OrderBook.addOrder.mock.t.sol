// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {OrderConfigV2, OrderV2, IO} from "src/interface/unstable/IOrderBookV3.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotRainMetaV1, META_MAGIC_NUMBER_V1} from "rain.metadata/IMetaV1.sol";
import {LibMeta} from "rain.metadata/LibMeta.sol";
import {IExpressionDeployerV3} from "rain.interpreter/src/interface/unstable/IExpressionDeployerV3.sol";

/// @title OrderBookAddOrderMockTest
/// @notice Tests the addOrder function of the OrderBook contract.
contract OrderBookAddOrderMockTest is OrderBookExternalMockTest {
    /// Adding an order without calculations MUST revert.
    function testAddOrderWithoutCalculationsReverts(address owner, OrderConfigV2 memory config) public {
        vm.prank(owner);
        config.evaluableConfig.bytecode = "";
        vm.expectRevert(abi.encodeWithSelector(OrderNoSources.selector, owner));
        iOrderbook.addOrder(config);
        (OrderV2 memory order, bytes32 orderHash) =
            LibTestAddOrder.expectedOrder(owner, config, iInterpreter, iStore, address(0));
        (order);
        assertTrue(!iOrderbook.orderExists(orderHash));
    }

    /// Adding an order without inputs MUST revert.
    function testAddOrderWithoutInputsReverts(address owner, OrderConfigV2 memory config) public {
        vm.prank(owner);
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        config.validInputs = new IO[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoInputs.selector, owner));
        iOrderbook.addOrder(config);
        (OrderV2 memory order, bytes32 orderHash) =
            LibTestAddOrder.expectedOrder(owner, config, iInterpreter, iStore, address(0));
        (order);
        assertTrue(!iOrderbook.orderExists(orderHash));
    }

    /// Adding an order without outputs MUST revert.
    function testAddOrderWithoutOutputsReverts(address owner, OrderConfigV2 memory config) public {
        vm.prank(owner);
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        config.validOutputs = new IO[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoOutputs.selector, owner));
        iOrderbook.addOrder(config);
        (OrderV2 memory order, bytes32 orderHash) =
            LibTestAddOrder.expectedOrder(owner, config, iInterpreter, iStore, address(0));
        (order);
        assertTrue(!iOrderbook.orderExists(orderHash));
    }

    /// Adding an order with calculations, inputs and outputs will succeed if
    /// the expression is valid according to the deployer. The resulting order
    /// MUST be emitted. This test assumes empty meta.
    function testAddOrderWithCalculationsInputsAndOutputsSucceeds(
        address owner,
        OrderConfigV2 memory config,
        address expression
    ) public {
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.meta = new bytes(0);
        (OrderV2 memory order, bytes32 orderhash) = addOrderWithChecks(owner, config, expression);
        (order);
        (orderhash);
    }

    /// Adding a valid order with a non-empty meta MUST revert if the meta is
    /// not self describing as a rain meta document.
    function testAddOrderWithNonEmptyMetaReverts(address owner, OrderConfigV2 memory config, address expression)
        public
    {
        vm.prank(owner);
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        vm.assume(!LibMeta.isRainMetaV1(config.meta));
        vm.assume(config.meta.length > 0);

        config.evaluableConfig.deployer = iDeployer;
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(iInterpreter, iStore, expression, "")
        );
        vm.expectRevert(abi.encodeWithSelector(NotRainMetaV1.selector, config.meta));
        iOrderbook.addOrder(config);

        (OrderV2 memory order, bytes32 orderHash) =
            LibTestAddOrder.expectedOrder(owner, config, iInterpreter, iStore, expression);
        (order);
        assertTrue(!iOrderbook.orderExists(orderHash));
    }

    /// Adding a valid order with a non-empty meta MUST emit MetaV1 if the meta
    /// is self describing as a rain meta document.
    function testAddOrderWithNonEmptyMetaEmitsMetaV1(address owner, OrderConfigV2 memory config, address expression)
        public
    {
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        vm.assume(config.meta.length > 0);

        // This is a bit of a hack, but it's the easiest way to get a valid
        // meta document.
        config.meta = abi.encodePacked(META_MAGIC_NUMBER_V1, config.meta);

        (OrderV2 memory order, bytes32 orderHash) = addOrderWithChecks(owner, config, expression);
        (order);
        (orderHash);
    }

    /// Alice and Bob can add orders with the same config. The resulting orders
    /// MUST be different.
    function testAddOrderTwoAccountsWithSameConfig(
        address alice,
        address bob,
        OrderConfigV2 memory config,
        address expression
    ) public {
        vm.assume(alice != bob);
        LibTestAddOrder.conformConfig(config, iDeployer);
        (OrderV2 memory aliceOrder, bytes32 aliceOrderHash) = addOrderWithChecks(alice, config, expression);
        (OrderV2 memory bobOrder, bytes32 bobOrderHash) = addOrderWithChecks(bob, config, expression);
        (aliceOrder);
        (bobOrder);
        assertTrue(aliceOrderHash != bobOrderHash);
    }

    /// Alice and Bob can add orders with different configs. The resulting orders
    /// MUST be different.
    function testAddOrderTwoAccountsWithDifferentConfig(
        address alice,
        address bob,
        OrderConfigV2 memory aliceConfig,
        OrderConfigV2 memory bobConfig,
        address aliceExpression,
        address bobExpression
    ) public {
        vm.assume(alice != bob);
        LibTestAddOrder.conformConfig(aliceConfig, iDeployer);
        LibTestAddOrder.conformConfig(bobConfig, iDeployer);
        (OrderV2 memory aliceOrder, bytes32 aliceOrderHash) = addOrderWithChecks(alice, aliceConfig, aliceExpression);
        (OrderV2 memory bobOrder, bytes32 bobOrderHash) = addOrderWithChecks(bob, bobConfig, bobExpression);
        (aliceOrder);
        (bobOrder);
        assertTrue(aliceOrderHash != bobOrderHash);
    }

    /// Alice can add orders with different configs. The resulting orders MUST
    /// be different.
    function testAddOrderSameAccountWithDifferentConfig(
        address alice,
        OrderConfigV2 memory configOne,
        OrderConfigV2 memory configTwo,
        address expressionOne,
        address expressionTwo
    ) public {
        LibTestAddOrder.conformConfig(configOne, iDeployer);
        LibTestAddOrder.conformConfig(configTwo, iDeployer);
        (OrderV2 memory expectedOrderOne, bytes32 expectedOrderOneHash) =
            LibTestAddOrder.expectedOrder(alice, configOne, iInterpreter, iStore, expressionOne);
        (OrderV2 memory expectedOrderTwo, bytes32 expectedOrderTwoHash) =
            LibTestAddOrder.expectedOrder(alice, configTwo, iInterpreter, iStore, expressionTwo);
        (expectedOrderOne);
        (expectedOrderTwo);
        assertTrue(expectedOrderOneHash != expectedOrderTwoHash);
        (OrderV2 memory orderOne, bytes32 orderOneHash) = addOrderWithChecks(alice, configOne, expressionOne);
        (OrderV2 memory orderTwo, bytes32 orderTwoHash) = addOrderWithChecks(alice, configTwo, expressionTwo);
        (orderOne);
        (orderTwo);
        assertTrue(orderOneHash != orderTwoHash);
    }
}
