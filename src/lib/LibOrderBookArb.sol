// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {TaskV2} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibOrderBook} from "./LibOrderBook.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

/// Thrown when the stack is not empty after the access control dispatch.
error NonZeroBeforeArbStack();

/// Thrown when the lender is not the trusted `OrderBook`.
/// @param badLender The untrusted lender calling `onFlashLoan`.
error BadLender(address badLender);

library LibOrderBookArb {
    using SafeERC20 for IERC20;

    function finalizeArb(
        TaskV2 memory task,
        address ordersInputToken,
        uint8 inputDecimals,
        address ordersOutputToken,
        uint8 outputDecimals
    ) internal {
        bytes32[][] memory context = new bytes32[][](1);
        bytes32[] memory col = new bytes32[](3);

        {
            // Send all unspent input tokens to the sender.
            uint256 inputBalance = IERC20(ordersInputToken).balanceOf(address(this));
            if (inputBalance > 0) {
                IERC20(ordersInputToken).safeTransfer(msg.sender, inputBalance);
            }
            (Float input, bool lossless) = LibDecimalFloat.fromFixedDecimalLossyPacked(inputBalance, inputDecimals);
            (lossless);
            col[0] = Float.unwrap(input);
        }

        {
            // Send all unspent output tokens to the sender.
            uint256 outputBalance = IERC20(ordersOutputToken).balanceOf(address(this));
            if (outputBalance > 0) {
                IERC20(ordersOutputToken).safeTransfer(msg.sender, outputBalance);
            }

            (Float output, bool lossless) = LibDecimalFloat.fromFixedDecimalLossyPacked(outputBalance, outputDecimals);
            (lossless);
            col[1] = Float.unwrap(output);
        }

        {
            // Send any remaining gas to the sender.
            // Slither false positive here. We want to send everything to the sender
            // because this contract should be empty of all gas and tokens between
            // uses. Anyone who sends tokens or gas to an arb contract without
            // calling `arb` is going to lose their tokens/gas.
            // See https://github.com/crytic/slither/issues/1658
            uint256 gasBalance = address(this).balance;
            Address.sendValue(payable(msg.sender), gasBalance);
            col[2] = Float.unwrap(LibDecimalFloat.packLossless(int256(gasBalance), -18));
        }

        context[0] = col;

        TaskV2[] memory post = new TaskV2[](1);
        post[0] = task;
        LibOrderBook.doPost(context, post);
    }
}
