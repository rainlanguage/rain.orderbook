// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {IERC3156FlashBorrower, ON_FLASH_LOAN_CALLBACK_SUCCESS} from "../interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC3156FlashLender} from "../interface/ierc3156/IERC3156FlashLender.sol";

/// Thrown when the `onFlashLoan` callback returns anything other than
/// ON_FLASH_LOAN_CALLBACK_SUCCESS.
/// @param result The value that was returned by `onFlashLoan`.
error FlashLenderCallbackFailed(bytes32 result);

/// @dev Flash fee is always 0 for orderbook as there's no entity to take
/// revenue for `Orderbook` and its more important anyway that flashloans happen
/// to connect external liquidity to live orders via arbitrage.
uint256 constant FLASH_FEE = 0;

/// @title OrderBookV3FlashLender
/// @notice Implements `IERC3156FlashLender` for `OrderBook`. Based on the
/// reference implementation by Alberto Cuesta Cañada found at
/// https://eips.ethereum.org/EIPS/eip-3156#flash-loan-reference-implementation
abstract contract OrderBookV3FlashLender is IERC3156FlashLender {
    using SafeERC20 for IERC20;

    /// @inheritdoc IERC3156FlashLender
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        override
        returns (bool)
    {
        IERC20(token).safeTransfer(address(receiver), amount);

        bytes32 result = receiver.onFlashLoan(msg.sender, token, amount, FLASH_FEE, data);
        if (result != ON_FLASH_LOAN_CALLBACK_SUCCESS) {
            revert FlashLenderCallbackFailed(result);
        }

        IERC20(token).safeTransferFrom(address(receiver), address(this), amount + FLASH_FEE);

        return true;
    }

    /// @inheritdoc IERC3156FlashLender
    function flashFee(address, uint256) external pure override returns (uint256) {
        return FLASH_FEE;
    }

    /// There's no limit to the size of a flash loan from `Orderbook` other than
    /// the current tokens deposited in `Orderbook`. If there is an active debt
    /// then loans are disabled so the max becomes `0` until after repayment.
    /// @inheritdoc IERC3156FlashLender
    function maxFlashLoan(address token) external view override returns (uint256) {
        return IERC20(token).balanceOf(address(this));
    }
}
