// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {RouteProcessorOrderBookV4ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV4ArbOrderTakerTest.sol";
import {
    OrderV3,
    EvaluableV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibNamespace, DEFAULT_STATE_NAMESPACE, WrongEvaluable} from "src/abstract/OrderBookV4ArbCommon.sol";
import {RouteProcessorOrderBookV4ArbOrderTaker} from "src/concrete/arb/RouteProcessorOrderBookV4ArbOrderTaker.sol";

contract RouteProcessorOrderBookV4ArbOrderTakerExpressionTest is RouteProcessorOrderBookV4ArbOrderTakerTest {
    function expression() internal virtual override returns (bytes memory) {
        // We're going to test with a mock so it doesn't matter what the expression is.
        return hex"deadbeef";
    }

    function testRouteProcessorTakeOrdersWrongExpression(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        EvaluableV3 memory evaluable
    ) public {
        vm.assume(
            address(evaluable.interpreter) != address(iInterpreter) || evaluable.store != iInterpreterStore
                || keccak256(evaluable.bytecode) != keccak256(expression())
        );
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(WrongEvaluable.selector));
        RouteProcessorOrderBookV4ArbOrderTaker(iArb).arb2(
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            0,
            evaluable
        );
    }

    function testRouteProcessorTakeOrdersExpression(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256[] memory stack,
        uint256[] memory kvs
    ) public {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.mockCall(
            address(iInterpreter),
            abi.encodeWithSelector(
                IInterpreterV3.eval3.selector,
                iInterpreterStore,
                LibNamespace.qualifyNamespace(DEFAULT_STATE_NAMESPACE, address(iArb))
            ),
            abi.encode(stack, kvs)
        );
        vm.expectCall(
            address(iInterpreter),
            abi.encodeWithSelector(
                IInterpreterV3.eval3.selector,
                iInterpreterStore,
                LibNamespace.qualifyNamespace(DEFAULT_STATE_NAMESPACE, address(iArb))
            )
        );

        if (kvs.length > 0) {
            vm.mockCall(
                address(iInterpreterStore),
                abi.encodeWithSelector(IInterpreterStoreV2.set.selector, DEFAULT_STATE_NAMESPACE, kvs),
                abi.encode("")
            );
            vm.expectCall(
                address(iInterpreterStore),
                abi.encodeWithSelector(IInterpreterStoreV2.set.selector, DEFAULT_STATE_NAMESPACE, kvs)
            );
        }

        RouteProcessorOrderBookV4ArbOrderTaker(iArb).arb2(
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            0,
            EvaluableV3(iInterpreter, iInterpreterStore, expression())
        );
    }
}
