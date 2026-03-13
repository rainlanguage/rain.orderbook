// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {EvaluableV4, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV4.sol";
import {
    IInterpreterV4,
    SourceIndexV2,
    DEFAULT_STATE_NAMESPACE
} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IRaindexV6, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {LibEvaluable} from "rain.interpreter.interface/lib/caller/LibEvaluable.sol";

/// @param orderBook The `OrderBook` contract to arb against.
/// @param task The task to run as post for each arb.
/// @param implementationData The constructor data for the specific
/// implementation of the arb contract.
struct OrderBookV6ArbConfig {
    address orderBook;
    TaskV2 task;
    bytes implementationData;
}

/// Thrown when the task does not match the expected hash.
error WrongTask();

/// @dev "Before arb" is evaluated before the arb is executed. Ostensibly
/// allows for some kind of access control to the arb.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);

/// @title OrderBookV6ArbCommon
/// @notice Common base for arb contracts that interact with `OrderBook`.
/// Stores a task hash at construction time and validates it on each arb call
/// via the `onlyValidTask` modifier.
abstract contract OrderBookV6ArbCommon {
    using LibEvaluable for EvaluableV4;

    /// @notice Emitted on construction with the full config.
    /// @param sender The deployer address.
    /// @param config The arb config used to construct this contract.
    event Construct(address sender, OrderBookV6ArbConfig config);

    /// @notice Hash of the configured task, or `bytes32(0)` if no task was set.
    /// Used by `onlyValidTask` to gate arb calls.
    bytes32 public immutable iTaskHash = 0;

    /// @param config The arb config for this contract.
    constructor(OrderBookV6ArbConfig memory config) {
        // Emit events before any external calls are made.
        emit Construct(msg.sender, config);

        if (config.task.evaluable.bytecode.length != 0) {
            iTaskHash = keccak256(abi.encode(config.task));
        }
    }

    /// @notice Reverts with `WrongTask` if `iTaskHash` is nonzero and does not
    /// match the hash of the provided task. Passes through if no task was
    /// configured at construction.
    modifier onlyValidTask(TaskV2 memory task) {
        if (iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))) {
            revert WrongTask();
        }
        _;
    }
}
