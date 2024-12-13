// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {LibOrder, OrderV3} from "src/lib/LibOrder.sol";
import {LibOrderBook} from "src/lib/LibOrderBook.sol";
import {TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

/// @title LibOrderTest
/// Exercises the LibOrder library.
contract LibOrderTest is Test {
    /// Hashing should always produce the same result for the same input.
    /// forge-config: default.fuzz.runs = 100
    function testHashEqual(OrderV3 memory a) public pure {
        assertTrue(LibOrder.hash(a) == LibOrder.hash(a));
    }

    /// Hashing should always produce different results for different inputs.
    /// forge-config: default.fuzz.runs = 100
    function testHashNotEqual(OrderV3 memory a, OrderV3 memory b) public pure {
        assertTrue(LibOrder.hash(a) != LibOrder.hash(b));
    }

    function doPost(uint256[][] memory context, TaskV1[] memory post) external {
        LibOrderBook.doPost(context, post);
    }
}
