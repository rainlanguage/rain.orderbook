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
import {
    RouteProcessorOrderBookV5ArbOrderTaker,
    OrderBookV5ArbConfig
} from "src/concrete/arb/RouteProcessorOrderBookV5ArbOrderTaker.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract RouteProcessorOrderBookV5ArbOrderTakerSenderTest is RouteProcessorOrderBookV5ArbOrderTakerTest {
    /// forge-config: default.fuzz.runs = 100
    function testRouteProcessorTakeOrdersSender(OrderV4 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        RouteProcessorOrderBookV5ArbOrderTaker(iArb).arb4(
            iOrderBook,
            TakeOrdersConfigV4(
                Float(0, 0), Float(type(int256).max, 0), Float(type(int256).max, 0), orders, abi.encode(bytes("0x00"))
            ),
            TaskV2({
                evaluable: EvaluableV4(iInterpreter, iInterpreterStore, ""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
