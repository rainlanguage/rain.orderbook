// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {LibEncodedDispatch, EncodedDispatch} from "rain.interpreter.interface/lib/caller/LibEncodedDispatch.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";
import {ON_FLASH_LOAN_CALLBACK_SUCCESS} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IOrderBookV3, TakeOrdersConfigV2, NoOrders} from "rain.orderbook.interface/interface/IOrderBookV3.sol";
import {
    IInterpreterV2,
    SourceIndexV2,
    DEFAULT_STATE_NAMESPACE
} from "rain.interpreter.interface/interface/IInterpreterV2.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {BadLender, MinimumOutput, NonZeroBeforeArbStack, OrderBookV3ArbConfigV1} from "./OrderBookV3ArbCommon.sol";
import {EvaluableConfigV3, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";

/// Thrown when the initiator is not the order book.
/// @param badInitiator The untrusted initiator of the flash loan.
error BadInitiator(address badInitiator);

/// Thrown when the flash loan fails somehow.
error FlashLoanFailed();

/// Thrown when the swap fails.
error SwapFailed();

/// Thrown when "Before arb" expects inputs.
error NonZeroBeforeArbInputs();

/// @dev "Before arb" is evaluated before the flash loan is taken. Ostensibly
/// allows for some kind of access control to the arb.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);
/// @dev "Before arb" has no inputs.
uint256 constant BEFORE_ARB_MIN_INPUTS = 0;
/// @dev "Before arb" has no outputs.
uint256 constant BEFORE_ARB_MIN_OUTPUTS = 0;
/// @dev "Before arb" has no outputs.
uint256 constant BEFORE_ARB_MAX_OUTPUTS = 0;

/// @title OrderBookV3FlashBorrower
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
abstract contract OrderBookV3FlashBorrower is IERC3156FlashBorrower, ReentrancyGuard, ERC165 {
    using Address for address;
    using SafeERC20 for IERC20;

    event Construct(address sender, OrderBookV3ArbConfigV1 config);

    /// `OrderBook` contract to lend and arb against.
    IOrderBookV3 public immutable iOrderBook;

    /// The encoded dispatch that will run for access control to `arb`.
    EncodedDispatch public immutable iI9rDispatch;
    /// The interpreter that will eval access control to `arb`.
    IInterpreterV2 public immutable iI9r;
    /// The associated store for the interpreter.
    IInterpreterStoreV2 public immutable iI9rStore;

    constructor(OrderBookV3ArbConfigV1 memory config) {
        // @todo This could be paramaterised on `arb`.
        iOrderBook = IOrderBookV3(config.orderBook);

        // Emit events before any external calls are made.
        emit Construct(msg.sender, config);

        IInterpreterV2 i9r = IInterpreterV2(address(0));
        IInterpreterStoreV2 i9rStore = IInterpreterStoreV2(address(0));
        EncodedDispatch i9rDispatch = EncodedDispatch.wrap(0);

        // If there are sources to eval then initialize the dispatch, otherwise
        // it will remain 0 and we can skip evaluation on `arb`.
        if (LibBytecode.sourceCount(config.evaluableConfig.bytecode) > 0) {
            address expression;

            bytes memory io;
            // We have to trust the deployer because it produces the expression
            // address for the dispatch anyway.
            // All external functions on this contract have `onlyNotInitializing`
            // modifier on them so can't be reentered here anyway.
            //slither-disable-next-line reentrancy-benign
            (i9r, i9rStore, expression, io) = config.evaluableConfig.deployer.deployExpression2(
                config.evaluableConfig.bytecode, config.evaluableConfig.constants
            );
            // There can't be any inputs because we don't pass any in.
            uint256 inputs;
            assembly ("memory-safe") {
                inputs := and(mload(add(io, 1)), 0xFF)
            }
            if (inputs > 0) {
                revert NonZeroBeforeArbInputs();
            }
            i9rDispatch = LibEncodedDispatch.encode2(expression, BEFORE_ARB_SOURCE_INDEX, BEFORE_ARB_MAX_OUTPUTS);
        }

        iI9r = i9r;
        iI9rStore = i9rStore;
        iI9rDispatch = i9rDispatch;
    }

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
    function _exchange(TakeOrdersConfigV2 memory takeOrders, bytes memory exchangeData) internal virtual {}

    /// @inheritdoc IERC3156FlashBorrower
    function onFlashLoan(address initiator, address, uint256, uint256, bytes calldata data)
        external
        returns (bytes32)
    {
        // As per reference implementation.
        if (msg.sender != address(iOrderBook)) {
            revert BadLender(msg.sender);
        }
        // As per reference implementation.
        if (initiator != address(this)) {
            revert BadInitiator(initiator);
        }

        (TakeOrdersConfigV2 memory takeOrders, bytes memory exchangeData) =
            abi.decode(data, (TakeOrdersConfigV2, bytes));

        // Dispatch the `_exchange` hook to ensure we have the correct asset
        // type and amount to fill the orders.
        _exchange(takeOrders, exchangeData);

        // At this point `exchange` should have sent the tokens required to match
        // the orders so take orders now.
        // We don't do anything with the total input/output amounts here because
        // the flash loan itself will take back what it needs, and we simply
        // keep anything left over according to active balances.
        (uint256 totalInput, uint256 totalOutput) = iOrderBook.takeOrders(takeOrders);
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
    /// @param takeOrders As per `IOrderBookV3.takeOrders`.
    /// @param minimumSenderOutput The minimum output that must be sent to the
    /// sender by the end of the arb call. This, in combination with the
    /// orderbook's own asset handling, is expected to REPLACE the standard
    /// slippage protection that would be provided by a DEX. The sender is
    /// expected to calculate absolute values based on prevailing conditions
    /// such as gas price and the risk of holding the assets any arb profit is
    /// denominated in.
    /// @param exchangeData Arbitrary bytes that will be passed to `_exchange`
    /// after the flash loan is taken. The inheriting contract is responsible
    /// for decoding this data and defining how it controls interactions with
    /// the external liquidity. For example, `GenericPoolOrderBookV4FlashBorrower`
    /// uses this data as a literal encoded external call.
    function arb(TakeOrdersConfigV2 calldata takeOrders, uint256 minimumSenderOutput, bytes calldata exchangeData)
        external
        payable
        nonReentrant
    {
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

        // We can't repay more than the minimum that the orders are going to
        // give us and there's no reason to borrow less.
        uint256 flashLoanAmount = takeOrders.minimumInput;

        // Run the access control dispatch if it is set.
        EncodedDispatch dispatch = iI9rDispatch;
        if (EncodedDispatch.unwrap(dispatch) > 0) {
            (uint256[] memory stack, uint256[] memory kvs) = iI9r.eval2(
                iI9rStore,
                LibNamespace.qualifyNamespace(DEFAULT_STATE_NAMESPACE, address(this)),
                dispatch,
                LibContext.build(new uint256[][](0), new SignedContextV1[](0)),
                new uint256[](0)
            );
            // This can only happen if the interpreter is broken.
            if (stack.length > 0) {
                revert NonZeroBeforeArbStack();
            }
            // Persist any state changes from the expression.
            if (kvs.length > 0) {
                iI9rStore.set(DEFAULT_STATE_NAMESPACE, kvs);
            }
        }

        // Take the flash loan, which will in turn call `onFlashLoan`, which is
        // expected to process an exchange against external liq to pay back the
        // flash loan, cover the orders and remain in profit.
        IERC20(ordersInputToken).safeApprove(address(iOrderBook), 0);
        IERC20(ordersInputToken).safeApprove(address(iOrderBook), type(uint256).max);
        if (!iOrderBook.flashLoan(this, ordersOutputToken, flashLoanAmount, data)) {
            revert FlashLoanFailed();
        }
        IERC20(ordersInputToken).safeApprove(address(iOrderBook), 0);

        // Send all unspent input tokens to the sender.
        uint256 inputBalance = IERC20(ordersInputToken).balanceOf(address(this));
        if (inputBalance < minimumSenderOutput) {
            revert MinimumOutput(minimumSenderOutput, inputBalance);
        }
        if (inputBalance > 0) {
            IERC20(ordersInputToken).safeTransfer(msg.sender, inputBalance);
        }
        // Send all unspent output tokens to the sender.
        uint256 outputBalance = IERC20(ordersOutputToken).balanceOf(address(this));
        if (outputBalance > 0) {
            IERC20(ordersOutputToken).safeTransfer(msg.sender, outputBalance);
        }

        // Send any remaining gas to the sender.
        // Slither false positive here. We want to send everything to the sender
        // because the borrower contract should be empty of all gas and tokens
        // between uses. Anyone who sends tokens or gas to an arb contract
        // without calling `arb` is going to lose their tokens/gas.
        // See https://github.com/crytic/slither/issues/1658
        Address.sendValue(payable(msg.sender), address(this).balance);
    }
}
