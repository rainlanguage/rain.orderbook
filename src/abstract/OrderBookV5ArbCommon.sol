// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
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

/// Configuration for an arb contract to construct.
/// @param orderBook The `OrderBook` contract to arb against.
/// @param tasks The tasks to use as post for each arb.
/// @param implementationData The constructor data for the specific
/// implementation of the arb contract.
struct OrderBookV5ArbConfig {
    address orderBook;
    TaskV2 task;
    bytes implementationData;
}

/// Thrown when the task does not match the expected hash.
error WrongTask();

/// @dev "Before arb" is evaluated before the flash loan is taken. Ostensibly
/// allows for some kind of access control to the arb.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);

abstract contract OrderBookV5ArbCommon {
    using LibEvaluable for EvaluableV3;

    event Construct(address sender, OrderBookV5ArbConfig config);

    bytes32 public immutable iTaskHash = 0;

    constructor(OrderBookV5ArbConfig memory config) {
        // Emit events before any external calls are made.
        emit Construct(msg.sender, config);

        if (config.task.evaluable.bytecode.length != 0) {
            iTaskHash = keccak256(abi.encode(config.task));
        }
    }

    modifier onlyValidTask(TaskV1 memory task) {
        if (iTaskHash != bytes32(0) && iTaskHash != keccak256(abi.encode(task))) {
            revert WrongTask();
        }
        _;
    }
}
