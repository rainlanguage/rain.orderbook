// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {SourceIndexV2} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {
    EncodedDispatch,
    LibEncodedDispatch
} from "rain.interpreter.interface/lib/deprecated/caller/LibEncodedDispatch.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {IOrderBookV4, NoOrders} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {
    IOrderBookV5,
    IOrderBookV5ArbOrderTaker,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5ArbOrderTaker.sol";
import {IInterpreterV3, DEFAULT_STATE_NAMESPACE} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {TakeOrdersConfigV3} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {OrderBookV4ArbConfigV2, EvaluableV3, OrderBookV4ArbCommon, SignedContextV1} from "./OrderBookV4ArbCommon.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";
import {LibOrderBook} from "../lib/LibOrderBook.sol";
import {LibOrderBookArb, NonZeroBeforeArbStack, BadLender} from "../lib/LibOrderBookArb.sol";

/// Thrown when "before arb" wants inputs that we don't have.
error NonZeroBeforeArbInputs(uint256 inputs);

/// @dev "Before arb" is evaluabled before the arb is executed. Ostensibly this
/// is to allow for access control to the arb, the return values are ignored.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);

abstract contract OrderBookV5ArbOrderTaker is
    IOrderBookV5ArbOrderTaker,
    ReentrancyGuard,
    ERC165,
    OrderBookV4ArbCommon
{
    using SafeERC20 for IERC20;

    constructor(OrderBookV4ArbConfigV2 memory config) OrderBookV4ArbCommon(config) {}

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return (interfaceId == type(IOrderBookV4OrderTaker).interfaceId)
            || (interfaceId == type(IOrderBookV4ArbOrderTakerV2).interfaceId) || super.supportsInterface(interfaceId);
    }

    /// @inheritdoc IOrderBookV4ArbOrderTakerV2
    function arb3(IOrderBookV4 orderBook, TakeOrdersConfigV3 calldata takeOrders, TaskV2 calldata task)
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
        (uint256 totalInput, uint256 totalOutput) = orderBook.takeOrders2(takeOrders);
        (totalInput, totalOutput);
        IERC20(ordersInputToken).safeApprove(address(orderBook), 0);

        LibOrderBookArb.finalizeArb(task, ordersInputToken, ordersOutputToken);
    }

    /// @inheritdoc IOrderBookV4OrderTaker
    function onTakeOrders(address, address, uint256, uint256, bytes calldata) public virtual override {}
}
