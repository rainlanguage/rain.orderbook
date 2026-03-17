// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV5ArbOrderTaker,
    OrderBookV5ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV5ArbOrderTaker.sol";

contract GenericPoolOrderBookV5ArbOrderTakerTest is ArbTest {
    function buildArb(OrderBookV5ArbConfig memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV5ArbOrderTaker(config));
    }

    constructor() ArbTest() {}
}
