// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV1
} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";
import {
    OrderV3,
    EvaluableV3,
    IExpressionDeployerV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

contract GenericPoolOrderBookV4ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV4ArbConfigV1 memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV4ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
