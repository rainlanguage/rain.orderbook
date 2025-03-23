// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RouteProcessorOrderBookV5ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV5ArbOrderTakerTest.sol";
import {
    OrderV3,
    EvaluableV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2,
    TaskV1,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibNamespace, DEFAULT_STATE_NAMESPACE, WrongTask} from "src/abstract/OrderBookV5ArbCommon.sol";
import {RouteProcessorOrderBookV5ArbOrderTaker} from "src/concrete/arb/RouteProcessorOrderBookV5ArbOrderTaker.sol";
import {StateNamespace, FullyQualifiedNamespace} from "rain.interpreter.interface/interface/IInterpreterV3.sol";

contract RouteProcessorOrderBookV5ArbOrderTakerExpressionTest is RouteProcessorOrderBookV5ArbOrderTakerTest {
    function expression() internal virtual override returns (bytes memory) {
        // We're going to test with a mock so it doesn't matter what the expression is.
        return hex"deadbeef";
    }

    /// forge-config: default.fuzz.runs = 100
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

        vm.expectRevert(abi.encodeWithSelector(WrongTask.selector));
        RouteProcessorOrderBookV5ArbOrderTaker(iArb).arb4(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            TaskV1({evaluable: evaluable, signedContext: new SignedContextV1[](0)})
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testRouteProcessorTakeOrdersExpression(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256[] memory stack,
        uint256[] memory kvs
    ) public {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        StateNamespace ns = StateNamespace.wrap(uint256(uint160(address(this))));
        FullyQualifiedNamespace fqns = LibNamespace.qualifyNamespace(ns, address(iArb));

        vm.mockCall(
            address(iInterpreter),
            abi.encodeWithSelector(IInterpreterV3.eval3.selector, iInterpreterStore, fqns),
            abi.encode(stack, kvs)
        );
        vm.expectCall(
            address(iInterpreter), abi.encodeWithSelector(IInterpreterV3.eval3.selector, iInterpreterStore, fqns)
        );

        if (kvs.length > 0) {
            vm.mockCall(
                address(iInterpreterStore), abi.encodeWithSelector(IInterpreterStoreV2.set.selector, ns), abi.encode("")
            );
            vm.expectCall(address(iInterpreterStore), abi.encodeWithSelector(IInterpreterStoreV2.set.selector, ns));
        }

        RouteProcessorOrderBookV5ArbOrderTaker(iArb).arb4(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            TaskV1({
                evaluable: EvaluableV3(iInterpreter, iInterpreterStore, expression()),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
