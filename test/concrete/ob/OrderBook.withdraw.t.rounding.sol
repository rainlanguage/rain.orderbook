// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {Reenteroor, IERC20} from "test/util/concrete/Reenteroor.sol";
import {TaskV2} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibDecimalFloatImplementation} from "rain.math.float/lib/implementation/LibDecimalFloatImplementation.sol";

/// @title OrderBookWithdrawTestRounding
/// Tests withdrawing from the order book with rounding issue for withdraw amount.
contract OrderBookWithdrawTestRounding is OrderBookExternalMockTest {
    using Math for uint256;

    using LibDecimalFloat for Float;
    using LibDecimalFloatImplementation for Float;

    /// forge-config: default.fuzz.runs = 100
    function testWithdrawRoundingNoDeposit(
       address alice,
       bytes32 vaultId,
       uint256 withdrawAmount18,
       uint256 decimals
    ) external {
        withdrawAmount18 = bound(withdrawAmount18, 1, type(uint256).max);
        decimals = bound(decimals, 0, 18);
        
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), 0),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.approve.selector, address(iOrderbook), type(uint256).max),
            abi.encode(true)
        );
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(decimals)));
        vm.mockCall(
            address(iToken0), 
            abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount18), 
            abi.encode(true)
        );
        
        vm.expectEmit(false, false, false, true);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, uint8(decimals));
        emit WithdrawV2(alice, address(iToken0), vaultId, withdrawAmount, Float.wrap(0), 0);
        
        TaskV2[] memory task = new TaskV2[](0);
        vm.prank(alice);
        iOrderbook.withdraw3(address(iToken0), vaultId, withdrawAmount, task);
        vm.stopPrank();
        
        // Should still have zero balance since no deposit was made
        assertTrue(iOrderbook.vaultBalance2(address(alice), address(iToken0), vaultId).isZero(), "vault balance should be zero");
    }

    
}
