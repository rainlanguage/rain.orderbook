// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {RouteProcessorOrderBookV4ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV4ArbOrderTakerTest.sol";
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
import {
    RouteProcessorOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV2
} from "src/concrete/arb/RouteProcessorOrderBookV4ArbOrderTaker.sol";

contract RouteProcessorOrderBookV4ArbOrderTakerSenderTest is RouteProcessorOrderBookV4ArbOrderTakerTest {
    /// forge-config: default.fuzz.runs = 100
    function testRouteProcessorTakeOrdersSender(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        RouteProcessorOrderBookV4ArbOrderTaker(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))),
            TaskV1({
                evaluable: EvaluableV3(iInterpreter, iInterpreterStore, ""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
