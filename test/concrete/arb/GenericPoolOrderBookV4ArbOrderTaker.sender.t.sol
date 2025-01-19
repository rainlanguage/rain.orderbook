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
    TaskV1,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

contract GenericPoolOrderBookV4ArbOrderTakerSenderTest is GenericPoolOrderBookV4ArbOrderTakerTest {
    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolTakeOrdersSender(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV4ArbOrderTaker(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            TaskV1({
                evaluable: EvaluableV3(iInterpreter, iInterpreterStore, ""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
