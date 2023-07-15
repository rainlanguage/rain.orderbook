// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "test/util/abstract/OrderBookExternalTest.sol";

/// @title OrderBookAddOrderTest
/// @notice Tests the addOrder function of the OrderBook contract.
contract OrderBookAddOrderTest is OrderBookExternalTest {
    /// Adding an order without calculations MUST revert.
    function testAddOrderWithoutCalculationsReverts(OrderConfig memory config) public {
        config.evaluableConfig.sources = new bytes[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoSources.selector, address(this)));
        iOrderbook.addOrder(config);
    }

    /// Adding an order without inputs MUST revert.
    function testAddOrderWithoutInputsReverts(OrderConfig memory config) public {
        vm.assume(config.evaluableConfig.sources.length > 1);
        config.validInputs = new IO[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoInputs.selector, address(this)));
        iOrderbook.addOrder(config);
    }

    /// Adding an order without outputs MUST revert.
    function testAddOrderWithoutOutputsReverts(OrderConfig memory config) public {
        vm.assume(config.evaluableConfig.sources.length > 1);
        vm.assume(config.validInputs.length > 0);
        config.validOutputs = new IO[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoOutputs.selector, address(this)));
        iOrderbook.addOrder(config);
    }

    /// Adding an order with calculations, inputs and outputs will succeed if
    /// the expression is valid according to the deployer. The resulting order
    /// MUST be emitted. This test assumes empty meta.
    function testAddOrderWithCalculationsInputsAndOutputsSucceeds(OrderConfig memory config, address expression)
        public
    {
        vm.assume(config.evaluableConfig.sources.length >= 2);
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.meta = new bytes(0);
        config.evaluableConfig.deployer = iDeployer;
        Evaluable memory expectedEvaluable = Evaluable(iInterpreter, iStore, expression);
        Order memory expectedOrder = Order(
            address(this),
            config.evaluableConfig.sources[1].length > 0,
            expectedEvaluable,
            config.validInputs,
            config.validOutputs
        );
        bytes32 expectedOrderHash = LibOrder.hash(expectedOrder);
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(iInterpreter, iStore, expression)
        );
        vm.expectEmit(false, false, false, true);
        emit AddOrder(address(this), iDeployer, expectedOrder, expectedOrderHash);
        vm.record();
        vm.recordLogs();
        iOrderbook.addOrder(config);
        // MetaV1 is NOT emitted if the meta is empty.
        assertEq(vm.getRecordedLogs().length, 1);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check, 1x for live write.
        assertEq(reads.length, 5);
        // 2x for reentrancy guard, 1x for live write.
        assertEq(writes.length, 3);

        // Adding the same order again MUST revert. This MAY be impossible to
        // encounter for a real expression deployer, as the deployer MAY NOT
        // return the same address twice, but it is possible to mock.
        vm.expectRevert(abi.encodeWithSelector(OrderExists.selector, address(this), expectedOrderHash));
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(iInterpreter, iStore, expression)
        );
        iOrderbook.addOrder(config);
    }
}
