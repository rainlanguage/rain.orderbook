// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

import {OrderBookV5ArbOrderTaker, OrderBookV5ArbConfig, Float} from "../../abstract/OrderBookV5ArbOrderTaker.sol";

contract GenericPoolOrderBookV5ArbOrderTaker is OrderBookV5ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(OrderBookV5ArbConfig memory config) OrderBookV5ArbOrderTaker(config) {}

    /// @inheritdoc OrderBookV5ArbOrderTaker
    function onTakeOrders2(
        address inputToken,
        address outputToken,
        Float inputAmountSent,
        Float totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders2(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        (address spender, address pool, bytes memory encodedFunctionCall) =
            abi.decode(takeOrdersData, (address, address, bytes));

        IERC20(inputToken).safeApprove(spender, 0);
        IERC20(inputToken).safeApprove(spender, type(uint256).max);
        bytes memory returnData = pool.functionCallWithValue(encodedFunctionCall, address(this).balance);
        // Nothing can be done with returnData as `takeOrders` does not support
        // it.
        (returnData);
        IERC20(inputToken).safeApprove(spender, 0);
    }

    /// Allow receiving gas.
    fallback() external {}
}
