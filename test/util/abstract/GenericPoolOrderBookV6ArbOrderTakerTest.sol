// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV6ArbOrderTaker,
    OrderBookV6ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol";

contract GenericPoolOrderBookV6ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV6ArbConfig memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV6ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
