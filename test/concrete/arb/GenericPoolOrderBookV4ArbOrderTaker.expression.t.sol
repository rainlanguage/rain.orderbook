// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {GenericPoolOrderBookV4ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV4ArbOrderTakerTest.sol";
import {
    GenericPoolOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV2
} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";
import {
    OrderV3,
    EvaluableV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2,
    SignedContextV1,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {
    LibNamespace,
    DEFAULT_STATE_NAMESPACE,
    BEFORE_ARB_SOURCE_INDEX,
    WrongTask
} from "src/abstract/OrderBookV4ArbCommon.sol";
import {CALCULATE_ORDER_ENTRYPOINT} from "src/concrete/ob/OrderBook.sol";
import {StateNamespace, FullyQualifiedNamespace} from "rain.interpreter.interface/interface/IInterpreterV3.sol";

contract GenericPoolOrderBookV4ArbOrderTakerExpressionTest is GenericPoolOrderBookV4ArbOrderTakerTest {
    function expression() internal virtual override returns (bytes memory) {
        // We're going to test with a mock so it doesn't matter what the expression is.
        return hex"deadbeef";
    }

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolTakeOrdersWrongExpression(
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
        GenericPoolOrderBookV4ArbOrderTaker(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            TaskV1({evaluable: evaluable, signedContext: new SignedContextV1[](0)})
        );
    }

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolTakeOrdersExpression(
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
                address(iInterpreterStore),
                abi.encodeWithSelector(IInterpreterStoreV2.set.selector, ns, kvs),
                abi.encode("")
            );
            vm.expectCall(address(iInterpreterStore), abi.encodeWithSelector(IInterpreterStoreV2.set.selector, ns, kvs));
        }

        GenericPoolOrderBookV4ArbOrderTaker(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            TaskV1({
                evaluable: EvaluableV3(iInterpreter, iInterpreterStore, expression()),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
