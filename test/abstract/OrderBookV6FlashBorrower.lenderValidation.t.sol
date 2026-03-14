// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {
    GenericPoolOrderBookV6FlashBorrower,
    OrderBookV6ArbConfig
} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {BadLender} from "../../src/abstract/OrderBookV6FlashBorrower.sol";
import {EvaluableV4, SignedContextV1, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {MaliciousLender} from "test/util/concrete/MaliciousLender.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";

contract OrderBookV6FlashBorrowerLenderValidationTest is Test {
    /// A malicious lender calls onFlashLoan directly. The call reverts with
    /// BadLender because msg.sender is not the configured orderbook.
    function testMaliciousLenderCannotExploitOnFlashLoan() external {
        MockToken token = new MockToken("Token", "TKN", 18);

        GenericPoolOrderBookV6FlashBorrower arb = new GenericPoolOrderBookV6FlashBorrower(
            OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                })
            )
        );

        MaliciousLender attacker = new MaliciousLender();

        vm.expectRevert(abi.encodeWithSelector(BadLender.selector, address(attacker)));
        attacker.attack(arb, address(token), 0, hex"");
    }
}
