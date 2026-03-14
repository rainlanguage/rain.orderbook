// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

import {
    OrderBookV6FlashBorrower,
    SafeERC20,
    IERC20,
    TakeOrdersConfigV5,
    OrderBookV6ArbConfig
} from "../../abstract/OrderBookV6FlashBorrower.sol";

/// @title GenericPoolOrderBookV6FlashBorrower
/// @notice Flash-loan arb that swaps via an arbitrary external pool call.
/// @dev Implements the OrderBookV6FlashBorrower interface for an external
/// liquidity source that behaves vaguely like a standard AMM. The
/// `exchangeData` from `arb` is decoded into a spender, pool and callData.
/// The `callData` is literally the encoded function call to the pool. This
/// allows the `arb` caller to process a trade against any liquidity source
/// that can swap tokens within a single function call.
/// The `spender` is the address that will be approved to spend the input token
/// on `takeOrders`, which is almost always going to be the pool itself. If you
/// are unsure, simply set it to the pool address.
contract GenericPoolOrderBookV6FlashBorrower is OrderBookV6FlashBorrower {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(OrderBookV6ArbConfig memory config) OrderBookV6FlashBorrower(config) {}

    /// @inheritdoc OrderBookV6FlashBorrower
    function _exchange(TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData) internal virtual override {
        (address spender, address pool, bytes memory encodedFunctionCall) =
            abi.decode(exchangeData, (address, address, bytes));

        address borrowedToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

        // Approve-call-revoke: the caller controls spender and pool, which is
        // safe because the contract holds no tokens or ETH between arb
        // operations — there is nothing for a malicious caller to extract.
        IERC20(borrowedToken).forceApprove(spender, type(uint256).max);
        // Nothing can be done with returnData as 3156 does not support it.
        //slither-disable-next-line unused-return
        pool.functionCallWithValue(encodedFunctionCall, address(this).balance);
        IERC20(borrowedToken).forceApprove(spender, 0);
    }

    /// Allow arbitrary calls and ETH transfers to this contract without
    /// reverting. Any ETH received is swept to msg.sender by finalizeArb.
    receive() external payable {}
    fallback() external payable {}
}
