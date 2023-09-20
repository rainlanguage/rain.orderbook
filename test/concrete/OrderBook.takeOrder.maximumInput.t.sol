// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookExternalRealTest, Vm} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    Order,
    SignedContextV1,
    TakeOrderConfig,
    TakeOrdersConfigV2,
    ZeroMaximumInput,
    IO,
    EvaluableConfigV2,
    OrderConfigV2
} from "src/interface/unstable/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter/src/interface/unstable/IParserV1.sol";

contract OrderBookTakeOrderMaximumInputTest is OrderBookExternalRealTest {
    /// If there is some live order(s) but the maxTakerInput is zero we error as
    /// the caller has full control over this, and it would cause none of the
    /// orders to be taken.
    function testTakeOrderNoopZeroMaxTakerInput(Order memory order, SignedContextV1 memory signedContext) external {
        vm.assume(order.validInputs.length > 0);
        vm.assume(order.validOutputs.length > 0);
        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        orders[0] = TakeOrderConfig(order, 0, 0, signedContexts);
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, 0, type(uint256).max, orders, "");
        vm.expectRevert(ZeroMaximumInput.selector);
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// Add an order with unlimited maximum output and take it with a maximum
    /// input. Only the maximum input should be taken.
    function testTakeOrderMaximumInput() external {
        address alice = address(uint160(uint256(keccak256("alice.rain.test"))));
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));
        uint256 vaultId = 0;

        Order memory order;

        {
            (bytes memory bytecode, uint256[] memory constants) =
                IParserV1(address(iDeployer)).parse("_ _:max-decimal18-value() 2e18;:;");
            IO[] memory inputs = new IO[](1);
            inputs[0] = IO(address(iToken0), 18, vaultId);
            IO[] memory outputs = new IO[](1);
            outputs[0] = IO(address(iToken1), 18, vaultId);
            EvaluableConfigV2 memory evaluableConfig = EvaluableConfigV2(iDeployer, bytecode, constants);
            OrderConfigV2 memory orderConfig = OrderConfigV2(inputs, outputs, evaluableConfig, "");

            vm.prank(alice);
            vm.recordLogs();
            iOrderbook.addOrder(orderConfig);
            Vm.Log[] memory entries = vm.getRecordedLogs();
            assertEq(entries.length, 3);
            (,, order,) = abi.decode(entries[2].data, (address, address, Order, bytes32));

            vm.prank(alice);
            // Deposit some large amount of output tokens.
            vm.mockCall(
                address(iToken1),
                abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), 1e50),
                abi.encode(true)
            );
            iOrderbook.deposit(address(iToken1), vaultId, 1e50);
        }

        uint256 expectedTakerInput = 1e5;
        uint256 expectedTakerOutput = 2e5;
        vm.prank(bob);
        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        orders[0] = TakeOrderConfig(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, expectedTakerInput, type(uint256).max, orders, "");

        // Mock and expect the token transfers.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transfer.selector, bob, expectedTakerInput),
            abi.encode(true)
        );
        vm.expectCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transfer.selector, bob, expectedTakerInput),
            1
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput),
            abi.encode(true)
        );
        vm.expectCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput),
            1
        );

        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        assertEq(totalTakerInput, expectedTakerInput);
        assertEq(totalTakerOutput, expectedTakerOutput);
    }
}
