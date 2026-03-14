// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {
    GenericPoolOrderBookV6ArbOrderTaker,
    OrderBookV6ArbConfig
} from "../../src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol";
import {EvaluableV4, SignedContextV1, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";

contract OrderBookV6ArbCommonITaskHashTest is Test {
    /// iTaskHash MUST equal keccak256(abi.encode(task)) when constructed with
    /// non-empty bytecode.
    function testITaskHashNonEmpty() external {
        TaskV2 memory task = TaskV2({
            evaluable: EvaluableV4(
                IInterpreterV4(address(0x1111)), IInterpreterStoreV3(address(0x2222)), hex"deadbeef"
            ),
            signedContext: new SignedContextV1[](0)
        });

        GenericPoolOrderBookV6ArbOrderTaker arb = new GenericPoolOrderBookV6ArbOrderTaker(OrderBookV6ArbConfig(task));

        assertEq(arb.iTaskHash(), keccak256(abi.encode(task)));
    }

    /// iTaskHash MUST be bytes32(0) when constructed with empty bytecode.
    function testITaskHashEmpty() external {
        TaskV2 memory task = TaskV2({
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            signedContext: new SignedContextV1[](0)
        });

        GenericPoolOrderBookV6ArbOrderTaker arb = new GenericPoolOrderBookV6ArbOrderTaker(OrderBookV6ArbConfig(task));

        assertEq(arb.iTaskHash(), bytes32(0));
    }
}
