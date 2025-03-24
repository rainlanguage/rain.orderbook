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
    TaskV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";

contract GenericPoolOrderBookV5ArbOrderTakerSenderTest is GenericPoolOrderBookV5ArbOrderTakerTest {
    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolTakeOrdersSender(OrderV4 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV5ArbOrderTaker(iArb).arb5(
            iOrderBook,
            TakeOrdersConfigV4(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            TaskV2({
                evaluable: EvaluableV4(iInterpreter, iInterpreterStore, ""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
