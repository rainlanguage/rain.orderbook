// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

/// @title OrderBookV4FlashLenderMaxFlashLoanTest
/// Tests the maximum flash loan amount for `OrderBookV4FlashLender`.
contract OrderBookV4FlashLenderMaxFlashLoanTest is OrderBookExternalMockTest {
    /// Tests that the maximum flash loan amount for `OrderBookV4FlashLender` is
    /// the balance of the token in the order book.
    function testFlashMaxLoan(uint256 amount) public {
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.balanceOf.selector, address(iOrderbook)), abi.encode(amount)
        );
        assertEq(iOrderbook.maxFlashLoan(address(iToken0)), amount);
    }
}
