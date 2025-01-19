// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
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
