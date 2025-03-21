// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    RouteProcessorOrderBookV5ArbOrderTaker,
    OrderBookV5ArbConfig
} from "src/concrete/arb/RouteProcessorOrderBookV5ArbOrderTaker.sol";

contract RouteProcessorOrderBookV5ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV5ArbConfig memory config) internal override returns (address) {
        return address(new RouteProcessorOrderBookV5ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
