// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

import {OrderBookV4ArbOrderTaker, OrderBookV4ArbConfigV2} from "../../abstract/OrderBookV4ArbOrderTaker.sol";

contract GenericPoolOrderBookV4ArbOrderTaker is OrderBookV4ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(OrderBookV4ArbConfigV2 memory config) OrderBookV4ArbOrderTaker(config) {}

    /// @inheritdoc OrderBookV4ArbOrderTaker
    function onTakeOrders(
        address inputToken,
        address outputToken,
        uint256 inputAmountSent,
        uint256 totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
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
