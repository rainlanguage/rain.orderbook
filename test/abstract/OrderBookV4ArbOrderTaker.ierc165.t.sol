// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {
    OrderBookV4ArbOrderTaker,
    IOrderBookV4ArbOrderTakerV2,
    EvaluableV3,
    OrderBookV4ArbConfigV2,
    IOrderBookV4OrderTaker,
    TaskV1,
    SignedContextV1
} from "src/abstract/OrderBookV4ArbOrderTaker.sol";
import {IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {ChildOrderBookV4ArbOrderTaker} from "../util/concrete/ChildOrderBookV4ArbOrderTaker.sol";

contract OrderBookV4ArbOrderTakerIERC165Test is Test {
    /// Test that ERC165 and IOrderBookV4ArbOrderTaker are supported interfaces
    /// as per ERC165.
    function testOrderBookV4ArbOrderTakerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV4ArbOrderTakerV2).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV4OrderTaker).interfaceId);

        ChildOrderBookV4ArbOrderTaker arbOrderTaker = new ChildOrderBookV4ArbOrderTaker();
        assertTrue(arbOrderTaker.supportsInterface(type(IERC165).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV4ArbOrderTakerV2).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV4OrderTaker).interfaceId));
        assertFalse(arbOrderTaker.supportsInterface(badInterfaceId));
    }
}
