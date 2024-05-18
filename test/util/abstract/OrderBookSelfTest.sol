// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "lib/forge-std/src/Test.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";

import {OrderBook} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookSelfTest
/// Abstract contract that is an `OrderBook` and can be used to test itself.
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Mocks all externalities during construction.
abstract contract OrderBookSelfTest is Test, OrderBook {}
