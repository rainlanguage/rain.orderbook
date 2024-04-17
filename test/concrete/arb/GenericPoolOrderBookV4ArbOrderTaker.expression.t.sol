// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {GenericPoolOrderBookV4ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV4ArbOrderTakerTest.sol";
import {
    GenericPoolOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV1
} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";
import {
    OrderV3,
    EvaluableV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {
    LibNamespace,
    DEFAULT_STATE_NAMESPACE,
    BEFORE_ARB_SOURCE_INDEX
} from "src/abstract/OrderBookV4ArbCommon.sol";

contract GenericPoolOrderBookV4ArbOrderTakerExpressionTest is GenericPoolOrderBookV4ArbOrderTakerTest {
    function expression() internal virtual override returns (bytes memory) {
        // We're going to test with a mock so it doesn't matter what the expression is.
        return hex"deadbeef";
    }

    function testGenericPoolTakeOrdersExpression(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex, uint256[] memory stack, uint256[] memory kvs)
        public
    {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.mockCall(
            address(iInterpreter),
            abi.encodeWithSelector(
                IInterpreterV3.eval3.selector,
                iInterpreterStore,
                LibNamespace.qualifyNamespace(DEFAULT_STATE_NAMESPACE, address(iArb)),
                expression(),
                BEFORE_ARB_SOURCE_INDEX,
                LibContext.build(new uint256[][](0), new SignedContextV1[](0)),
                new uint256[](0)
            ),
            abi.encode(stack, kvs)
        );

        if (kvs.length > 0) {
            vm.mockCall(
                address(iInterpreterStore),
                abi.encodeWithSelector(
                    IInterpreterStoreV2.set.selector,
                    DEFAULT_STATE_NAMESPACE,
                    kvs
                ),
                abi.encode("")
            );
        }
        GenericPoolOrderBookV4ArbOrderTaker(iArb).arb2(
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            0,
            EvaluableV3(iInterpreter, iInterpreterStore, expression())
        );
    }
}