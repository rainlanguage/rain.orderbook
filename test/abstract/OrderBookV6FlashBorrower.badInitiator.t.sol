// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {
    GenericPoolOrderBookV6FlashBorrower,
    OrderBookV6ArbConfig
} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {BadInitiator} from "../../src/abstract/OrderBookV6FlashBorrower.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";
import {EvaluableV4, SignedContextV1, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";

contract OrderBookV6FlashBorrowerBadInitiatorTest is Test {
    GenericPoolOrderBookV6FlashBorrower arb;

    constructor() {
        arb = new GenericPoolOrderBookV6FlashBorrower(
            OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                }),
                ""
            )
        );
    }

    /// onFlashLoan MUST revert with BadInitiator when called with an initiator
    /// that is not the flash borrower contract itself.
    function testOnFlashLoanBadInitiator(address badInitiator) external {
        vm.assume(badInitiator != address(arb));

        // Prank as the deterministic orderbook address to isolate the
        // BadInitiator check.
        vm.prank(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
        vm.expectRevert(abi.encodeWithSelector(BadInitiator.selector, badInitiator));
        arb.onFlashLoan(badInitiator, address(0), 0, 0, abi.encode(new bytes(0), new bytes(0)));
    }
}
