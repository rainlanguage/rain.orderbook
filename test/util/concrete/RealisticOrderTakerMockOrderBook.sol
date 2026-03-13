// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {TakeOrdersConfigV5, Float} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IRaindexV6OrderTaker} from "rain.raindex.interface/interface/IRaindexV6OrderTaker.sol";
import {MockOrderBookBase} from "test/util/abstract/MockOrderBookBase.sol";

/// @dev Mock orderbook with real takeOrders4 transfers and onTakeOrders2
/// callback, matching the real orderbook flow for order taker arb contracts.
contract RealisticOrderTakerMockOrderBook is MockOrderBookBase {
    using SafeERC20 for IERC20;
    function takeOrders4(TakeOrdersConfigV5 calldata config) external override returns (Float, Float) {
        address ordersInputToken = config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token;
        address ordersOutputToken = config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token;

        uint256 outputAmount = IERC20(ordersOutputToken).balanceOf(address(this));

        // Send ordersOutputToken to taker (taker's input).
        IERC20(ordersOutputToken).safeTransfer(msg.sender, outputAmount);

        // Callback: taker swaps received tokens for the tokens the OB will pull.
        IRaindexV6OrderTaker(msg.sender).onTakeOrders2(
            ordersOutputToken, ordersInputToken, Float.wrap(0), Float.wrap(0), config.data
        );

        // Pull ordersInputToken from taker.
        uint256 inputBalance = IERC20(ordersInputToken).balanceOf(msg.sender);
        IERC20(ordersInputToken).safeTransferFrom(msg.sender, address(this), inputBalance);

        return (Float.wrap(0), Float.wrap(0));
    }
}
