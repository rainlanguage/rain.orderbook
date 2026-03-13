// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {GenericPoolOrderBookV6ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV6ArbOrderTakerTest.sol";
import {GenericPoolOrderBookV6ArbOrderTaker} from "src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol";
import {ArbTest} from "test/util/abstract/ArbTest.sol";
import {
    GenericPoolOrderBookV6FlashBorrower,
    OrderBookV6ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";

/// @dev Tests fallback behavior on the order taker arb contract.
contract OrderBookV6ArbOrderTakerFallbackTest is GenericPoolOrderBookV6ArbOrderTakerTest {
    /// The fallback MUST accept calldata that does not match any selector.
    function testFallbackAcceptsCalldata() external {
        // 0xdeadbeef does not match any function selector on the arb contracts.
        (bool success,) = iArb.call(hex"deadbeef");
        assertTrue(success, "fallback should accept non-matching calldata");
    }

    /// The fallback MUST accept empty calldata.
    function testFallbackAcceptsEmptyCalldata() external {
        (bool success,) = iArb.call("");
        assertTrue(success, "fallback should accept empty calldata");
    }

    /// The fallback MUST reject ETH transfers (non-payable).
    function testFallbackRejectsETH() external {
        vm.deal(address(this), 1 ether);
        (bool success,) = iArb.call{value: 1}("");
        assertFalse(success, "fallback should reject ETH");
    }
}

/// @dev Tests fallback behavior on the flash borrower arb contract.
contract OrderBookV6FlashBorrowerFallbackTest is ArbTest {
    function buildArb(OrderBookV6ArbConfig memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV6FlashBorrower(config));
    }

    constructor() ArbTest() {}

    /// The fallback MUST accept calldata that does not match any selector.
    function testFallbackAcceptsCalldata() external {
        (bool success,) = iArb.call(hex"deadbeef");
        assertTrue(success, "fallback should accept non-matching calldata");
    }

    /// The fallback MUST accept empty calldata.
    function testFallbackAcceptsEmptyCalldata() external {
        (bool success,) = iArb.call("");
        assertTrue(success, "fallback should accept empty calldata");
    }

    /// The fallback MUST reject ETH transfers (non-payable).
    function testFallbackRejectsETH() external {
        vm.deal(address(this), 1 ether);
        (bool success,) = iArb.call{value: 1}("");
        assertFalse(success, "fallback should reject ETH");
    }
}
