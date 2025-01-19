// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    RouteProcessorOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV2
} from "src/concrete/arb/RouteProcessorOrderBookV4ArbOrderTaker.sol";

contract RouteProcessorOrderBookV4ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV4ArbConfigV2 memory config) internal override returns (address) {
        return address(new RouteProcessorOrderBookV4ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
