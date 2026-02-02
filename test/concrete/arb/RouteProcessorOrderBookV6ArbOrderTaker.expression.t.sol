// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    RouteProcessorOrderBookV6ArbOrderTakerTest
} from "test/util/abstract/RouteProcessorOrderBookV6ArbOrderTakerTest.sol";
import {
    OrderV4,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";
import {WrongTask} from "src/abstract/OrderBookV6ArbCommon.sol";
import {RouteProcessorOrderBookV6ArbOrderTaker} from "src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
import {
    StateNamespace,
    FullyQualifiedNamespace
} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

contract RouteProcessorOrderBookV6ArbOrderTakerExpressionTest is RouteProcessorOrderBookV6ArbOrderTakerTest {
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
        RouteProcessorOrderBookV6ArbOrderTaker(iArb)
            .arb5(
                iOrderBook,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(iRefundoor, iRefundoor, "")
            }),
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

        RouteProcessorOrderBookV6ArbOrderTaker(iArb)
            .arb5(
                iOrderBook,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(iRefundoor, iRefundoor, "")
            }),
                TaskV2({
                evaluable: EvaluableV4(iInterpreter, iInterpreterStore, expression()),
                signedContext: new SignedContextV1[](0)
            })
            );
    }
}
