// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    RouteProcessorOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV1
} from "src/concrete/arb/RouteProcessorOrderBookV4ArbOrderTaker.sol";

contract RouteProcessorOrderBookV4ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV4ArbConfigV1 memory config) internal override returns (address) {
        return address(new RouteProcessorOrderBookV4ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
