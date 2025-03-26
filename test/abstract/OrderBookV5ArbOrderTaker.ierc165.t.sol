// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {
    OrderBookV5ArbOrderTaker,
    IOrderBookV5ArbOrderTaker,
    EvaluableV4,
    OrderBookV5ArbConfig,
    IOrderBookV5OrderTaker,
    TaskV2,
    SignedContextV1
} from "src/abstract/OrderBookV5ArbOrderTaker.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/unstable/IInterpreterStoreV3.sol";
import {ChildOrderBookV5ArbOrderTaker} from "../util/concrete/ChildOrderBookV5ArbOrderTaker.sol";

contract OrderBookV5ArbOrderTakerIERC165Test is Test {
    /// Test that ERC165 and IOrderBookV5ArbOrderTaker are supported interfaces
    /// as per ERC165.
    function testOrderBookV5ArbOrderTakerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV5ArbOrderTaker).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV5OrderTaker).interfaceId);

        ChildOrderBookV5ArbOrderTaker arbOrderTaker = new ChildOrderBookV5ArbOrderTaker();
        assertTrue(arbOrderTaker.supportsInterface(type(IERC165).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV5ArbOrderTaker).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV5OrderTaker).interfaceId));
        assertFalse(arbOrderTaker.supportsInterface(badInterfaceId));
    }
}
