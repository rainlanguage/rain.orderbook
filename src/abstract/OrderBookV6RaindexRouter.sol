// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/utils/ReentrancyGuard.sol";
import {ON_FLASH_LOAN_CALLBACK_SUCCESS} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {
    IOrderBookV6,
    TakeOrdersConfigV5,
    TaskV2,
    Float,
    QuoteV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {OrderBookV6ArbConfig, OrderBookV6ArbCommon} from "./OrderBookV6ArbCommon.sol";
import {LibOrderBookArb} from "../lib/LibOrderBookArb.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// Thrown when the initiator is not the order book.
/// @param badInitiator The untrusted initiator of the flash loan.
error BadInitiator(address badInitiator);

/// Thrown when the flash loan fails somehow.
error FlashLoanFailed();

/// Thrown when the swap fails.
error SwapFailed();

/// @title OrderBookV6RaindexRouter
/// @notice Abstract contract that liq-source specifialized contracts can inherit
/// to provide flash loan based routed arbitrage against external liquidity sources to
/// fill orderbook orders.
///
/// For example consider circuit:
///
/// start input = DAI
/// start output = USDC
/// external source router = USDC -> USDT
/// end input = USDT
/// end output = DAI
///
/// Assume external liq can exchange USDC to USDT, so 2 raindex orders can be traded
/// against eachother while their IO are NOT mirror:
///
/// - Flash loan 100 DAI from `Orderbook`
/// - Take the first order of the circuit (flash loan amount (DAI) goes as input and the order's output (USDC) is taken)
/// - Sell the first order's output (USDC) for market price through the external source (sushi, balancer, etc) for USDT
/// - Take the last order of the circuit (given the USDT as input and take its DAI as output)
/// - The circuit is now closed so we can repay the flash loan amount (DAI) and keep the profit (it can be both USDT and DAI)
abstract contract OrderBookV6RaindexRouter is IERC3156FlashBorrower, ReentrancyGuard, ERC165, OrderBookV6ArbCommon {
    using Address for address;
    using SafeERC20 for IERC20;

    constructor(OrderBookV6ArbConfig memory config) OrderBookV6ArbCommon(config) {}

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return interfaceId == type(IERC3156FlashBorrower).interfaceId || super.supportsInterface(interfaceId);
    }

    /// Hook that inheriting contracts MUST implement in order to achieve
    /// anything other than raising the ambient temperature of the room.
    /// `_exchange` is responsible for converting the flash loaned assets into
    /// the assets required to fill the orders. Generally this can only be
    /// achieved by interacting with an external liquidity source that is
    /// offering a better price than the orders require.
    /// @param takeOrders As per `arb`.
    /// @param exchangeData As per `arb`.
    // slither-disable-next-line dead-code
    function _exchange(TakeOrdersConfigV5[] memory takeOrders, bytes memory exchangeData) internal virtual {}

    /// @inheritdoc IERC3156FlashBorrower
    function onFlashLoan(address initiator, address, uint256, uint256, bytes calldata data)
        external
        returns (bytes32)
    {
        // As per reference implementation.
        if (initiator != address(this)) {
            revert BadInitiator(initiator);
        }

        (TakeOrdersConfigV5[] memory takeOrders, bytes memory exchangeData) =
            abi.decode(data, (TakeOrdersConfigV5[], bytes));

        // Dispatch the `_exchange` hook to ensure we have the correct asset
        // type and amount to fill the orders.
        _exchange(takeOrders, exchangeData);

        return ON_FLASH_LOAN_CALLBACK_SUCCESS;
    }

    /// Primary function to process arbitrage opportunities.
    /// Firstly the access gate is evaluated to ensure the sender is allowed to
    /// submit arbitrage. If there is no access control the sender should expect
    /// to be front run on the arb for any sufficiently profitable opportunity.
    /// This may be desirable in some cases, as the sender may simply want to
    /// be clearing the orderbook and they are expecting profit/utility from the
    /// orderbook strategies themselves somehow.
    ///
    /// Secondly the flash loan is taken and the `_exchange` hook is called to
    /// allow the inheriting contract to convert the flash loaned assets into
    /// the assets required to fill the orders.
    ///
    /// Finally the orders are taken and the remaining assets are sent to the
    /// sender.
    ///
    /// @param orderBook The orderbook address
    /// @param takeOrders As per `IOrderBookV5.takeOrders3`.
    /// @param exchangeData Arbitrary bytes that will be passed to `_exchange`
    /// after the flash loan is taken. The inheriting contract is responsible
    /// for decoding this data and defining how it controls interactions with
    /// the external liquidity. For example, `GenericPoolOrderBookV5FlashBorrower`
    /// uses this data as a literal encoded external call.
    function arb4(
        IOrderBookV6 orderBook,
        TakeOrdersConfigV5[] memory takeOrders,
        bytes calldata exchangeData,
        TaskV2 calldata task
    ) external payable nonReentrant onlyValidTask(task) {
        // Mimic what OB would do anyway if called with zero orders.
        require(takeOrders.length == 2, "Unexpected take orders config length");
        if (takeOrders[0].orders.length == 0 || takeOrders[1].orders.length == 0) {
            revert IOrderBookV6.NoOrders();
        }

        address startTakeOrdersInputToken =
            takeOrders[0].orders[0].order.validInputs[takeOrders[0].orders[0].inputIOIndex].token;
        address endTakeOrdersInputToken =
            takeOrders[1].orders[0].order.validInputs[takeOrders[1].orders[0].inputIOIndex].token;

        require(
            startTakeOrdersInputToken
                == takeOrders[1].orders[0].order.validOutputs[takeOrders[1].orders[0].outputIOIndex].token,
            "start and end orders IO do NOT close the route circuit"
        );

        uint8 startInputDecimals = LibTOFUTokenDecimals.safeDecimalsForToken(startTakeOrdersInputToken);
        uint8 endInputDecimals = LibTOFUTokenDecimals.safeDecimalsForToken(endTakeOrdersInputToken);

        // Take the flash loan, which will in turn call `onFlashLoan`, which is
        // expected to process an exchange against external liq to pay back the
        // flash loan, cover the orders and remain in profit.
        //
        // We take all the current balance of orderbook divided by 2 as loan,
        // that's because its the max possible crealable amount by flash loan,
        // because the loan is taken before any takeOrders4() is processed, the
        // loan goes for the input amount of first order of the circuit, and
        // orderbook needs to have balance left to finish the last takeOrders4(),
        // all this while the flash loan is still open (not repaid), after the
        // last takeOrder4() is processed then the flash loan can be repaid, in
        // order words half of the orderbook token balance is used for completing
        // the first takeOrders4() as a flash loan and half for the last as flash
        // loan repay
        uint256 flashLoanAmount = IERC20(startTakeOrdersInputToken).balanceOf(address(orderBook)) / 2;
        Float flashLoanAmountFloat = LibDecimalFloat.fromFixedDecimalLosslessPacked(flashLoanAmount, startInputDecimals);
        require(!LibDecimalFloat.isZero(flashLoanAmountFloat), "zero flash loan amount");

        // getting the last order's maxOuput, as the first order cannot clear
        // more than the maxOutput of the last order, the max possible clear
        // amount is min of maxOutput and flashLoanAmount
        //slither-disable-next-line unused-return
        (, Float maxOutput,) = orderBook.quote2(
            QuoteV2({
                order: takeOrders[1].orders[0].order,
                inputIOIndex: takeOrders[1].orders[0].inputIOIndex,
                outputIOIndex: takeOrders[1].orders[0].outputIOIndex,
                signedContext: takeOrders[1].orders[0].signedContext
            })
        );

        IERC20(startTakeOrdersInputToken).forceApprove(address(orderBook), 0);
        IERC20(startTakeOrdersInputToken).forceApprove(address(orderBook), type(uint256).max);

        // set max io
        if (LibDecimalFloat.gt(takeOrders[0].maximumIO, flashLoanAmountFloat)) {
            takeOrders[0].maximumIO = flashLoanAmountFloat;
        }
        if (LibDecimalFloat.gt(takeOrders[0].maximumIO, maxOutput)) {
            takeOrders[0].maximumIO = maxOutput;
        }
        takeOrders[0].IOIsInput = false; // must always be false

        bytes memory data = abi.encode(takeOrders, exchangeData);

        if (!orderBook.flashLoan(this, startTakeOrdersInputToken, flashLoanAmount, data)) {
            revert FlashLoanFailed();
        }
        IERC20(startTakeOrdersInputToken).forceApprove(address(orderBook), 0);

        LibOrderBookArb.finalizeArb(
            task, endTakeOrdersInputToken, endInputDecimals, startTakeOrdersInputToken, startInputDecimals
        );
    }
}
