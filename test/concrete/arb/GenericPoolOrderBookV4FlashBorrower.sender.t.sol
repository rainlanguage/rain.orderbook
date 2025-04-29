// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV5FlashBorrower,
    OrderBookV5ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV5FlashBorrower.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    EvaluableV4,
    TakeOrdersConfigV4,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract GenericPoolOrderBookV5FlashBorrowerTest is ArbTest {
    function buildArb(OrderBookV5ArbConfig memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV5FlashBorrower(config));
    }

    constructor() ArbTest() {}

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolOrderBookV5FlashBorrowerTakeOrdersSender(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex
    ) public {
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV5FlashBorrower(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV4(LibDecimalFloat.packLossless(0, 0), LibDecimalFloat.packLossless(type(int256).max, 0), LibDecimalFloat.packLossless(type(int256).max, 0), orders, ""),
            abi.encode(iRefundoor, iRefundoor, ""),
            TaskV2({
                evaluable: EvaluableV4(iInterpreter, iInterpreterStore, ""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
