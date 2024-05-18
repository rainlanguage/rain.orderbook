// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";

/// @title OrderBookV3FlashLenderFeeTest
/// Tests the fee charged by `OrderBookV3FlashLender`.
contract OrderBookV3FlashLenderFeeTest is OrderBookExternalMockTest {
    /// Tests that the fee charged by `OrderBookV3FlashLender` is 0.
    function testFlashFee(address token, uint256 amount) public {
        assertEq(iOrderbook.flashFee(token, amount), 0);
    }
}
