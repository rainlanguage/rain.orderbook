// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {GenericPoolOrderBookV5ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV5ArbOrderTakerTest.sol";
import {
    GenericPoolOrderBookV5ArbOrderTaker,
    OrderBookV5ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV5ArbOrderTaker.sol";
import {
    OrderV4,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV4,
    IInterpreterV4,
    IInterpreterStoreV3,
    SignedContextV1,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {
    LibNamespace,
    DEFAULT_STATE_NAMESPACE,
    BEFORE_ARB_SOURCE_INDEX,
    WrongTask
} from "src/abstract/OrderBookV5ArbCommon.sol";
import {CALCULATE_ORDER_ENTRYPOINT} from "src/concrete/ob/OrderBook.sol";
import {
    StateNamespace, FullyQualifiedNamespace
} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract GenericPoolOrderBookV5ArbOrderTakerExpressionTest is GenericPoolOrderBookV5ArbOrderTakerTest {
    function expression() internal virtual override returns (bytes memory) {
        // We're going to test with a mock so it doesn't matter what the expression is.
        return hex"deadbeef";
    }

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolTakeOrdersWrongExpression(
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
        GenericPoolOrderBookV5ArbOrderTaker(iArb).arb4(
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

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolTakeOrdersExpression(
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
                address(iInterpreterStore),
                abi.encodeWithSelector(IInterpreterStoreV3.set.selector, ns, kvs),
                abi.encode("")
            );
            vm.expectCall(address(iInterpreterStore), abi.encodeWithSelector(IInterpreterStoreV3.set.selector, ns, kvs));
        }

        GenericPoolOrderBookV5ArbOrderTaker(iArb).arb4(
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
