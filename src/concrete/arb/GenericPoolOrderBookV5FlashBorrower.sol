// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC3156FlashLender} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashLender.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

import {
    OrderBookV5FlashBorrower,
    SafeERC20,
    IERC20,
    Address,
    TakeOrdersConfigV4,
    OrderBookV5ArbConfig
} from "../../abstract/OrderBookV5FlashBorrower.sol";

/// @title GenericPoolOrderBookV5FlashBorrower
/// Implements the OrderBookV5FlashBorrower interface for an external liquidity
/// source that behaves vaguely like a standard AMM. The `exchangeData` from
/// `arb` is decoded into a spender, pool and callData. The `callData` is
/// literally the encoded function call to the pool. This allows the `arb`
/// caller to process a trade against any liquidity source that can swap tokens
/// within a single function call.
/// The `spender` is the address that will be approved to spend the input token
/// on `takeOrders`, which is almost always going to be the pool itself. If you
/// are unsure, simply set it to the pool address.
contract GenericPoolOrderBookV5FlashBorrower is OrderBookV5FlashBorrower {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(OrderBookV5ArbConfig memory config) OrderBookV5FlashBorrower(config) {}

    /// @inheritdoc OrderBookV5FlashBorrower
    function _exchange(TakeOrdersConfigV4 memory takeOrders, bytes memory exchangeData) internal virtual override {
        (address spender, address pool, bytes memory encodedFunctionCall) =
            abi.decode(exchangeData, (address, address, bytes));

        address borrowedToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

        IERC20(borrowedToken).safeApprove(spender, 0);
        IERC20(borrowedToken).safeApprove(spender, type(uint256).max);
        bytes memory returnData = pool.functionCallWithValue(encodedFunctionCall, address(this).balance);
        // Nothing can be done with returnData as 3156 does not support it.
        (returnData);
        IERC20(borrowedToken).safeApprove(spender, 0);
    }

    /// Allow receiving gas.
    fallback() external {}
}
