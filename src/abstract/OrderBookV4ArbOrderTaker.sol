// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {EvaluableConfigV3, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";
import {SourceIndexV2} from "rain.interpreter.interface/interface/IInterpreterV2.sol";
import {EncodedDispatch, LibEncodedDispatch} from "rain.interpreter.interface/lib/caller/LibEncodedDispatch.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {IOrderBookV4, NoOrders} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {
    IOrderBookV4ArbOrderTaker,
    IOrderBookV4OrderTaker
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4ArbOrderTaker.sol";
import {
    IInterpreterV3, DEFAULT_STATE_NAMESPACE
} from "rain.interpreter.interface/interface/unstable/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {TakeOrdersConfigV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {BadLender, MinimumOutput, NonZeroBeforeArbStack, OrderBookV4ArbConfigV1} from "./OrderBookV4ArbCommon.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";

/// Thrown when "before arb" wants inputs that we don't have.
error NonZeroBeforeArbInputs(uint256 inputs);

/// @dev "Before arb" is evaluabled before the arb is executed. Ostensibly this
/// is to allow for access control to the arb, the return values are ignored.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);
/// @dev "Before arb" has no return values.
uint256 constant BEFORE_ARB_MIN_OUTPUTS = 0;
/// @dev "Before arb" has no return values.
uint16 constant BEFORE_ARB_MAX_OUTPUTS = 0;

abstract contract OrderBookV4ArbOrderTaker is IOrderBookV4ArbOrderTaker, ReentrancyGuard, ERC165 {
    using SafeERC20 for IERC20;

    event Construct(address sender, OrderBookV4ArbConfigV1 config);

    IOrderBookV4 public immutable iOrderBook;
    EncodedDispatch public immutable iI9rDispatch;
    IInterpreterV2 public immutable iI9r;
    IInterpreterStoreV2 public immutable iI9rStore;

    constructor(OrderBookV4ArbConfigV1 memory config) {
        // @todo this could be paramaterised on `arb`.
        iOrderBook = IOrderBookV4(config.orderBook);

        // // Emit events before any external calls are made.
        emit Construct(msg.sender, config);

        IInterpreterV2 i9r = IInterpreterV2(address(0));
        IInterpreterStoreV2 i9rStore = IInterpreterStoreV2(address(0));
        EncodedDispatch i9rDispatch = EncodedDispatch.wrap(0);

        // If there are any sources to eval then initialize the dispatch,
        // otherwise it will remain 0 and we can skip evaluation on `arb`.
        if (LibBytecode.sourceCount(config.evaluableConfig.bytecode) > 0) {
            address expression;

            bytes memory io;
            // We have to trust the deployer because it produces the expression
            // address for dispatch anyway.
            // All external functions on this contract have `onlyNotInitializing`
            // modifier on them so can't be reentered here anyway.
            //slither-disable-next-line reentrancy-benign
            (i9r, i9rStore, expression, io) = config.evaluableConfig.deployer.deployExpression2(
                config.evaluableConfig.bytecode, config.evaluableConfig.constants
            );
            {
                uint256 inputs;
                assembly ("memory-safe") {
                    inputs := and(mload(add(io, 1)), 0xFF)
                }
                if (inputs != 0) {
                    revert NonZeroBeforeArbInputs(inputs);
                }
            }
            i9rDispatch = LibEncodedDispatch.encode2(expression, BEFORE_ARB_SOURCE_INDEX, BEFORE_ARB_MAX_OUTPUTS);
        }

        iI9r = i9r;
        iI9rStore = i9rStore;
        iI9rDispatch = i9rDispatch;
    }

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return (interfaceId == type(IOrderBookV4OrderTaker).interfaceId)
            || (interfaceId == type(IOrderBookV4ArbOrderTaker).interfaceId) || super.supportsInterface(interfaceId);
    }

    /// @inheritdoc IOrderBookV4ArbOrderTaker
    function arb2(TakeOrdersConfigV3 calldata takeOrders, uint256 minimumSenderOutput) external payable nonReentrant {
        // Mimic what OB would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert NoOrders();
        }

        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

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
            // This can only happen if interpreter is broken.
            if (stack.length > 0) {
                revert NonZeroBeforeArbStack();
            }
            // Persist any state changes from the expression.
            if (kvs.length > 0) {
                iI9rStore.set(DEFAULT_STATE_NAMESPACE, kvs);
            }
        }

        IERC20(ordersInputToken).safeApprove(address(iOrderBook), 0);
        IERC20(ordersInputToken).safeApprove(address(iOrderBook), type(uint256).max);
        (uint256 totalInput, uint256 totalOutput) = iOrderBook.takeOrders(takeOrders);
        (totalInput, totalOutput);
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
        // because this contract should be empty of all gas and tokens between
        // uses. Anyone who sends tokens or gas to an arb contract without
        // calling `arb` is going to lose their tokens/gas.
        // See https://github.com/crytic/slither/issues/1658
        Address.sendValue(payable(msg.sender), address(this).balance);
    }

    /// @inheritdoc IOrderBookV4OrderTaker
    function onTakeOrders(address, address, uint256, uint256, bytes calldata) public virtual override {
        if (msg.sender != address(iOrderBook)) {
            revert BadLender(msg.sender);
        }
    }
}
