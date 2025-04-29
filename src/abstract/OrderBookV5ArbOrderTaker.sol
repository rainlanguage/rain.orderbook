// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {SourceIndexV2} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {
    EncodedDispatch,
    LibEncodedDispatch
} from "rain.interpreter.interface/lib/deprecated/caller/LibEncodedDispatch.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {IOrderBookV5, NoOrders} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {
    IOrderBookV5,
    IOrderBookV5ArbOrderTaker,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5ArbOrderTaker.sol";
import {
    IInterpreterV4, DEFAULT_STATE_NAMESPACE
} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/unstable/IInterpreterStoreV3.sol";
import {TakeOrdersConfigV4, Float} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {OrderBookV5ArbConfig, EvaluableV4, OrderBookV5ArbCommon, SignedContextV1} from "./OrderBookV5ArbCommon.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";
import {LibOrderBook} from "../lib/LibOrderBook.sol";
import {LibOrderBookArb, NonZeroBeforeArbStack, BadLender} from "../lib/LibOrderBookArb.sol";
import {IOrderBookV5OrderTaker} from "rain.orderbook.interface/interface/unstable/IOrderBookV5OrderTaker.sol";

/// Thrown when "before arb" wants inputs that we don't have.
error NonZeroBeforeArbInputs(uint256 inputs);

/// @dev "Before arb" is evaluabled before the arb is executed. Ostensibly this
/// is to allow for access control to the arb, the return values are ignored.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);

abstract contract OrderBookV5ArbOrderTaker is
    IOrderBookV5OrderTaker,
    IOrderBookV5ArbOrderTaker,
    ReentrancyGuard,
    ERC165,
    OrderBookV5ArbCommon
{
    using SafeERC20 for IERC20;

    constructor(OrderBookV5ArbConfig memory config) OrderBookV5ArbCommon(config) {}

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return (interfaceId == type(IOrderBookV5OrderTaker).interfaceId)
            || (interfaceId == type(IOrderBookV5ArbOrderTaker).interfaceId) || super.supportsInterface(interfaceId);
    }

    /// @inheritdoc IOrderBookV5ArbOrderTaker
    function arb4(IOrderBookV5 orderBook, TakeOrdersConfigV4 calldata takeOrders, TaskV2 calldata task)
        external
        payable
        nonReentrant
        onlyValidTask(task)
    {
        // Mimic what OB would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert NoOrders();
        }

        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

        IERC20(ordersInputToken).safeApprove(address(orderBook), 0);
        IERC20(ordersInputToken).safeApprove(address(orderBook), type(uint256).max);
        (Float totalInput, Float totalOutput) = orderBook.takeOrders3(takeOrders);
        (totalInput, totalOutput);
        IERC20(ordersInputToken).safeApprove(address(orderBook), 0);

        LibOrderBookArb.finalizeArb(task, ordersInputToken, ordersOutputToken);
    }

    /// @inheritdoc IOrderBookV5OrderTaker
    function onTakeOrders2(address, address, Float, Float, bytes calldata) public virtual override {}
}
