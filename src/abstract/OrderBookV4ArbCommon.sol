// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {EvaluableV3, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";
import {
    IInterpreterV3,
    SourceIndexV2,
    DEFAULT_STATE_NAMESPACE
} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IOrderBookV4, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {LibEvaluable} from "rain.interpreter.interface/lib/caller/LibEvaluable.sol";

/// Thrown when the minimum output for the sender is not met after the arb.
/// @param minimum The minimum output expected by the sender.
/// @param actual The actual output that would be received by the sender.
error MinimumOutput(uint256 minimum, uint256 actual);

/// Thrown when the stack is not empty after the access control dispatch.
error NonZeroBeforeArbStack();

/// Thrown when the lender is not the trusted `OrderBook`.
/// @param badLender The untrusted lender calling `onFlashLoan`.
error BadLender(address badLender);

/// Configuration for an arb contract to construct.
/// @param orderBook The `OrderBook` contract to arb against.
/// @param tasks The tasks to use as post for each arb.
/// @param implementationData The constructor data for the specific
/// implementation of the arb contract.
struct OrderBookV4ArbConfigV2 {
    address orderBook;
    TaskV1 task;
    bytes implementationData;
}

/// Thrown when the tasks do not match the expected hash.
error WrongTasks();

/// @dev "Before arb" is evaluated before the flash loan is taken. Ostensibly
/// allows for some kind of access control to the arb.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);

abstract contract OrderBookV4ArbCommon {
    using LibEvaluable for EvaluableV3;

    event Construct(address sender, OrderBookV4ArbConfigV2 config);

    constructor(OrderBookV4ArbConfigV2 memory config) {
        // Emit events before any external calls are made.
        emit Construct(msg.sender, config);
    }
}
