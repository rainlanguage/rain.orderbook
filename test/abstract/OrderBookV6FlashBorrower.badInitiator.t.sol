// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV6FlashBorrower,
    OrderBookV6ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {BadInitiator} from "src/abstract/OrderBookV6FlashBorrower.sol";
import {
    EvaluableV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";

contract OrderBookV6FlashBorrowerBadInitiatorTest is ArbTest {
    function buildArb(OrderBookV6ArbConfig memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV6FlashBorrower(config));
    }

    constructor() ArbTest() {}

    /// onFlashLoan MUST revert with BadInitiator when called with an initiator
    /// that is not the flash borrower contract itself.
    function testOnFlashLoanBadInitiator(address badInitiator) external {
        vm.assume(badInitiator != iArb);

        vm.expectRevert(abi.encodeWithSelector(BadInitiator.selector, badInitiator));
        GenericPoolOrderBookV6FlashBorrower(iArb).onFlashLoan(
            badInitiator, address(0), 0, 0, abi.encode(new bytes(0), new bytes(0))
        );
    }
}
