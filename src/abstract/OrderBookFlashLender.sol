// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import "../interface/ierc3156/IERC3156FlashBorrower.sol";
import "../interface/ierc3156/IERC3156FlashLender.sol";

/// Thrown when `flashLoan` token is zero address.
error ZeroToken();

/// Thrown when `flashLoan` receiver is zero address.
error ZeroReceiver();

/// Thrown when the `onFlashLoan` callback returns anything other than
/// ON_FLASH_LOAN_CALLBACK_SUCCESS.
/// @param result The value that was returned by `onFlashLoan`.
error FlashLenderCallbackFailed(bytes32 result);

/// Thrown when more than one debt is attempted simultaneously.
/// @param receiver The receiver of the active debt.
/// @param token The token of the active debt.
/// @param amount The amount of the active debt.
error ActiveDebt(address receiver, address token, uint256 amount);

/// @dev Flash fee is always 0 for orderbook as there's no entity to take
/// revenue for `Orderbook` and its more important anyway that flashloans happen
/// to connect external liquidity to live orders via arbitrage.
uint256 constant FLASH_FEE = 0;

/// @title OrderBookFlashLender
/// @notice Implements `IERC3156FlashLender` for `OrderBook`. Based on the
/// reference implementation by Alberto Cuesta Cañada found at
/// https://eips.ethereum.org/EIPS/eip-3156
/// Several features found in the reference implementation are simplified or
/// hardcoded for `Orderbook`.
abstract contract OrderBookFlashLender is IERC3156FlashLender {
    using Math for uint256;
    using SafeERC20 for IERC20;

    IERC3156FlashBorrower private _sReceiver = IERC3156FlashBorrower(address(0));
    address private _sToken = address(0);
    uint256 private _sAmount = 0;

    function _isActiveDebt() internal view returns (bool) {
        return (address(_sReceiver) != address(0)) || (_sToken != address(0)) || (_sAmount != 0);
    }

    function _checkActiveDebt() internal view {
        if (_isActiveDebt()) {
            revert ActiveDebt(address(_sReceiver), _sToken, _sAmount);
        }
    }

    /// Whenever `Orderbook` sends tokens to any address it MUST first attempt
    /// to decrease any outstanding flash loans for that address. Consider the
    /// case that Alice deposits 100 TKN and she is the only depositor of TKN
    /// then flash borrows 100 TKN. If she attempts to withdraw 100 TKN during
    /// her `onFlashLoan` callback then `Orderbook`:
    ///
    /// - has 0 TKN balance to process the withdrawal
    /// - MUST process the withdrawal as Alice has the right to withdraw her
    /// balance at any time
    /// - Has the 100 TKN debt active under Alice
    ///
    /// In this case `Orderbook` can simply forgive Alice's 100 TKN debt instead
    /// of actually transferring any tokens. The withdrawal can decrease her
    /// vault balance by 100 TKN decoupled from needing to know whether a
    /// tranfer or forgiveness happened.
    ///
    /// The same logic applies to withdrawals as sending tokens during
    /// `takeOrders` as the reason for sending tokens is irrelevant, all that
    /// matters is that `Orderbook` prioritises debt repayments over external
    /// transfers.
    ///
    /// If there is an active debt that only partially eclipses the withdrawal
    /// then the debt will be fully repaid and the remainder transferred as a
    /// real token transfer.
    ///
    /// Note that Alice can still contrive a situation that causes `Orderbook`
    /// to attempt to send tokens that it does not have. If Alice can write a
    /// smart contract to trigger withdrawals she can flash loan 100% of the
    /// TKN supply in `Orderbook` and trigger her contract to attempt a
    /// withdrawal. For any normal ERC20 token this will fail and revert as the
    /// `Orderbook` cannot send tokens it does not have under any circumstances,
    /// but the scenario is worth being aware of for more exotic token
    /// behaviours that may not be supported.
    ///
    /// @param token The token being sent or for the debt being paid.
    /// @param receiver The receiver of the token or holder of the debt.
    /// @param sendAmount The amount to send or repay.
    function _decreaseFlashDebtThenSendToken(address token, address receiver, uint256 sendAmount) internal {
        // If this token transfer matches the active debt then prioritise
        // reducing debt over sending tokens.
        if (token == _sToken && receiver == address(_sReceiver)) {
            uint256 debtReduction = sendAmount.min(_sAmount);
            sendAmount -= debtReduction;

            // Even if this completely zeros the amount the debt is considered
            // active until the `flashLoan` also clears the token and recipient.
            _sAmount -= debtReduction;
        }

        if (sendAmount > 0) {
            IERC20(token).safeTransfer(receiver, sendAmount);
        }
    }

    /// @inheritdoc IERC3156FlashLender
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        override
        returns (bool)
    {
        // This prevents reentrancy, loans can be taken sequentially within a
        // transaction but not simultanously.
        _checkActiveDebt();

        // Set the active debt before transferring tokens to prevent reeentrancy.
        // The active debt is set beyond the scope of `flashLoan` to facilitate
        // early repayment via. `_decreaseFlashDebtThenSendToken`.
        {
            if (token == address(0)) {
                revert ZeroToken();
            }
            if (address(receiver) == address(0)) {
                revert ZeroReceiver();
            }
            _sToken = token;
            _sReceiver = receiver;
            _sAmount = amount;
            if (amount > 0) {
                IERC20(token).safeTransfer(address(receiver), amount);
            }
        }

        bytes32 result = receiver.onFlashLoan(msg.sender, token, amount, FLASH_FEE, data);
        if (result != ON_FLASH_LOAN_CALLBACK_SUCCESS) {
            revert FlashLenderCallbackFailed(result);
        }

        // Pull tokens before releasing the active debt to prevent a new loan
        // from being taken reentrantly during the repayment of the current loan.
        {
            // Sync local `amount_` with global `_amount` in case an early
            // repayment was made during the loan term via.
            // `_decreaseFlashDebtThenSendToken`.
            amount = _sAmount;
            if (amount > 0) {
                IERC20(token).safeTransferFrom(address(receiver), address(this), amount);
                _sAmount = 0;
            }

            // Both of these are required to fully clear the active debt and
            // allow new debts.
            _sReceiver = IERC3156FlashBorrower(address(0));
            _sToken = address(0);
        }

        // Guard against some bad code path that allowed an active debt to remain
        // at this point. Should be impossible.
        _checkActiveDebt();

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
        return _isActiveDebt() ? 0 : IERC20(token).balanceOf(address(this));
    }
}
