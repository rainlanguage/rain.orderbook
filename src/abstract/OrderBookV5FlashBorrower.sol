// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {
    LibEncodedDispatch,
    EncodedDispatch
} from "rain.interpreter.interface/lib/deprecated/caller/LibEncodedDispatch.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";
import {ON_FLASH_LOAN_CALLBACK_SUCCESS} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {
    IOrderBookV5,
    TakeOrdersConfigV4,
    NoOrders,
    TaskV2,
    Float
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/unstable/IInterpreterStoreV3.sol";
import {OrderBookV5ArbConfig, OrderBookV5ArbCommon} from "./OrderBookV5ArbCommon.sol";
import {EvaluableV4, SignedContextV1} from "rain.interpreter.interface/interface/unstable/IInterpreterCallerV4.sol";
import {LibOrderBook} from "../lib/LibOrderBook.sol";
import {LibOrderBookArb, NonZeroBeforeArbStack, BadLender} from "../lib/LibOrderBookArb.sol";
import {LibTOFUTokenDecimals, TOFUTokenDecimals, TOFUOutcome} from "../lib/LibTOFUTokenDecimals.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// Thrown when the initiator is not the order book.
/// @param badInitiator The untrusted initiator of the flash loan.
error BadInitiator(address badInitiator);

/// Thrown when the flash loan fails somehow.
error FlashLoanFailed();

/// Thrown when the swap fails.
error SwapFailed();

/// @title OrderBookV5FlashBorrower
/// @notice Abstract contract that liq-source specifialized contracts can inherit
/// to provide flash loan based arbitrage against external liquidity sources to
/// fill orderbook orders.
///
/// For example consider a simple order:
///
/// input = DAI
/// output = USDT
/// IORatio = 1.01e18
/// Order amount = 100e18
///
/// Assume external liq is offering 102 DAI per USDT so it exceeds the IO ratio
/// but the order itself has no way to interact with the external contract.
/// The `OrderBookFlashBorrower` can:
///
/// - Flash loan 100 USDT from `Orderbook`
/// - Sell the 100 USDT for 102 DAI on external liq
/// - Take the order, giving 101 DAI and paying down 100 USDT loan
/// - Keep 1 DAI profit
///
/// As this contract is expected to be cloned using a minimal proxy there will
/// be many copies of it in the wild. Each copy can be access gated by a Rain
/// expression that is evaluated before anything else happens in `arb`. There
/// are many reasons why this might be desirable:
/// - Regulatory reasons that restrict how an arb bot operator can interact with
///   orders and/or external liq.
/// - The arb operator wants to attempt to prevent front running by other bots.
/// - The arb operator may prefer a dedicated instance of the contract to make
///   it easier to track profits, etc.
abstract contract OrderBookV5FlashBorrower is IERC3156FlashBorrower, ReentrancyGuard, ERC165, OrderBookV5ArbCommon {
    using Address for address;
    using SafeERC20 for IERC20;

    constructor(OrderBookV5ArbConfig memory config) OrderBookV5ArbCommon(config) {}

    mapping(address token => TOFUTokenDecimals tofuTokenDecimals) internal sTOFUTokenDecimals;

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
    //slither-disable-next-line dead-code
    function _exchange(TakeOrdersConfigV4 memory takeOrders, bytes memory exchangeData) internal virtual {}

    /// @inheritdoc IERC3156FlashBorrower
    function onFlashLoan(address initiator, address, uint256, uint256, bytes calldata data)
        external
        returns (bytes32)
    {
        // As per reference implementation.
        if (initiator != address(this)) {
            revert BadInitiator(initiator);
        }

        (TakeOrdersConfigV4 memory takeOrders, bytes memory exchangeData) =
            abi.decode(data, (TakeOrdersConfigV4, bytes));

        // Dispatch the `_exchange` hook to ensure we have the correct asset
        // type and amount to fill the orders.
        _exchange(takeOrders, exchangeData);

        // At this point `exchange` should have sent the tokens required to match
        // the orders so take orders now.
        // We don't do anything with the total input/output amounts here because
        // the flash loan itself will take back what it needs, and we simply
        // keep anything left over according to active balances.
        (Float totalInput, Float totalOutput) = IOrderBookV5(msg.sender).takeOrders3(takeOrders);
        (totalInput, totalOutput);

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
    /// @param takeOrders As per `IOrderBookV5.takeOrders3`.
    /// @param exchangeData Arbitrary bytes that will be passed to `_exchange`
    /// after the flash loan is taken. The inheriting contract is responsible
    /// for decoding this data and defining how it controls interactions with
    /// the external liquidity. For example, `GenericPoolOrderBookV5FlashBorrower`
    /// uses this data as a literal encoded external call.
    function arb3(
        IOrderBookV5 orderBook,
        TakeOrdersConfigV4 calldata takeOrders,
        bytes calldata exchangeData,
        TaskV2 calldata task
    ) external payable nonReentrant onlyValidTask(task) {
        // Mimic what OB would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert NoOrders();
        }

        // Encode everything that will be used by the flash loan callback.
        bytes memory data = abi.encode(takeOrders, exchangeData);
        // The token we receive from taking the orders is what we will use to
        // repay the flash loan.
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;
        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;

        (TOFUOutcome inputOutcome, uint8 inputDecimals) =
            LibTOFUTokenDecimals.decimalsForToken(sTOFUTokenDecimals, ordersInputToken);
        (inputOutcome);

        // We can't repay more than the minimum that the orders are going to
        // give us and there's no reason to borrow less.
        uint256 flashLoanAmount = LibDecimalFloat.toFixedDecimalLossless(
            takeOrders.minimumInput, inputDecimals
        );

        // Take the flash loan, which will in turn call `onFlashLoan`, which is
        // expected to process an exchange against external liq to pay back the
        // flash loan, cover the orders and remain in profit.
        IERC20(ordersInputToken).safeApprove(address(orderBook), 0);
        IERC20(ordersInputToken).safeApprove(address(orderBook), type(uint256).max);
        if (!orderBook.flashLoan(this, ordersOutputToken, flashLoanAmount, data)) {
            revert FlashLoanFailed();
        }
        IERC20(ordersInputToken).safeApprove(address(orderBook), 0);

        LibOrderBookArb.finalizeArb(task, ordersInputToken, ordersOutputToken);
    }
}
