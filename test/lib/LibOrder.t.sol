// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {LibOrder, OrderV2} from "src/lib/LibOrder.sol";

/// @title LibOrderTest
/// Exercises the LibOrder library.
contract LibOrderTest is Test {
    /// Hashing should always produce the same result for the same input.
    function testHashEqual(OrderV2 memory a) public {
        assertTrue(LibOrder.hash(a) == LibOrder.hash(a));
    }

    /// Hashing should always produce different results for different inputs.
    function testHashNotEqual(OrderV2 memory a, OrderV2 memory b) public {
        assertTrue(LibOrder.hash(a) != LibOrder.hash(b));
    }
}
