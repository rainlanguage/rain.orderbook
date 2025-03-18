// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV5FlashBorrower,
    OrderBookV5ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV5FlashBorrower.sol";
import {
    OrderV3,
    TakeOrderConfigV3,
    EvaluableV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2,
    TaskV1,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

contract GenericPoolOrderBookV5FlashBorrowerTest is ArbTest {
    function buildArb(OrderBookV5ArbConfig memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV5FlashBorrower(config));
    }

    constructor() ArbTest() {}

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolOrderBookV5FlashBorrowerTakeOrdersSender(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex
    ) public {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV5FlashBorrower(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, ""),
            abi.encode(iRefundoor, iRefundoor, ""),
            TaskV1({
                evaluable: EvaluableV3(iInterpreter, iInterpreterStore, ""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
