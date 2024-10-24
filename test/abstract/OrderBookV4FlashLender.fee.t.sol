// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";

/// @title OrderBookV4FlashLenderFeeTest
/// Tests the fee charged by `OrderBookV4FlashLender`.
contract OrderBookV4FlashLenderFeeTest is OrderBookExternalMockTest {
    /// Tests that the fee charged by `OrderBookV4FlashLender` is 0.
    function testFlashFee(address token, uint256 amount) public view {
        assertEq(iOrderbook.flashFee(token, amount), 0);
    }
}
