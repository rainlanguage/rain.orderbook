// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV2
} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";

contract GenericPoolOrderBookV4ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV4ArbConfigV2 memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV4ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
