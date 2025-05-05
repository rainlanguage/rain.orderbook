// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RouteProcessorOrderBookV5ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV5ArbOrderTakerTest.sol";
import {
    OrderV4,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV4,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibNamespace, DEFAULT_STATE_NAMESPACE, WrongTask} from "src/abstract/OrderBookV5ArbCommon.sol";
import {RouteProcessorOrderBookV5ArbOrderTaker} from "src/concrete/arb/RouteProcessorOrderBookV5ArbOrderTaker.sol";
import {
    StateNamespace, FullyQualifiedNamespace
} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract RouteProcessorOrderBookV5ArbOrderTakerExpressionTest is RouteProcessorOrderBookV5ArbOrderTakerTest {
    function expression() internal virtual override returns (bytes memory) {
        // We're going to test with a mock so it doesn't matter what the expression is.
        return hex"deadbeef";
    }

    /// forge-config: default.fuzz.runs = 100
    function testRouteProcessorTakeOrdersWrongExpression(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        EvaluableV4 memory evaluable
    ) public {
        vm.assume(
            address(evaluable.interpreter) != address(iInterpreter) || evaluable.store != iInterpreterStore
                || keccak256(evaluable.bytecode) != keccak256(expression())
        );
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(WrongTask.selector));
        RouteProcessorOrderBookV5ArbOrderTaker(iArb).arb4(
            iOrderBook,
            TakeOrdersConfigV4(
                LibDecimalFloat.packLossless(0, 0),
                LibDecimalFloat.packLossless(type(int224).max, 0),
                LibDecimalFloat.packLossless(type(int224).max, 0),
                orders,
                abi.encode(iRefundoor, iRefundoor, "")
            ),
            TaskV2({evaluable: evaluable, signedContext: new SignedContextV1[](0)})
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testRouteProcessorTakeOrdersExpression(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256[] memory stack,
        uint256[] memory kvs
    ) public {
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        StateNamespace ns = StateNamespace.wrap(uint256(uint160(address(this))));
        FullyQualifiedNamespace fqns = LibNamespace.qualifyNamespace(ns, address(iArb));

        vm.mockCall(
            address(iInterpreter), abi.encodeWithSelector(IInterpreterV4.eval4.selector), abi.encode(stack, kvs)
        );
        vm.expectCall(address(iInterpreter), abi.encodeWithSelector(IInterpreterV4.eval4.selector));

        if (kvs.length > 0) {
            vm.mockCall(
                address(iInterpreterStore), abi.encodeWithSelector(IInterpreterStoreV3.set.selector, ns), abi.encode("")
            );
            vm.expectCall(address(iInterpreterStore), abi.encodeWithSelector(IInterpreterStoreV3.set.selector, ns));
        }

        RouteProcessorOrderBookV5ArbOrderTaker(iArb).arb4(
            iOrderBook,
            TakeOrdersConfigV4(
                LibDecimalFloat.packLossless(0, 0),
                LibDecimalFloat.packLossless(type(int224).max, 0),
                LibDecimalFloat.packLossless(type(int224).max, 0),
                orders,
                abi.encode(iRefundoor, iRefundoor, "")
            ),
            TaskV2({
                evaluable: EvaluableV4(iInterpreter, iInterpreterStore, expression()),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
