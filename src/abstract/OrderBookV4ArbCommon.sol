// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {EvaluableV3, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";
import {
    IInterpreterV3,
    SourceIndexV2,
    DEFAULT_STATE_NAMESPACE
} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IOrderBookV4} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
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
/// @param evaluable The `EvaluableV3` to use as a pre-hook for each arb.
/// @param implementationData The constructor data for the specific
/// implementation of the arb contract.
struct OrderBookV4ArbConfigV1 {
    address orderBook;
    EvaluableV3 evaluable;
    bytes implementationData;
}

/// Thrown when the evaluable does not match the expected hash.
error WrongEvaluable();

/// @dev "Before arb" is evaluated before the flash loan is taken. Ostensibly
/// allows for some kind of access control to the arb.
SourceIndexV2 constant BEFORE_ARB_SOURCE_INDEX = SourceIndexV2.wrap(0);

abstract contract OrderBookV4ArbCommon {
    using LibEvaluable for EvaluableV3;

    event Construct(address sender, OrderBookV4ArbConfigV1 config);

    bytes32 public immutable iEvaluableHash;

    constructor(OrderBookV4ArbConfigV1 memory config) {
        // Emit events before any external calls are made.
        emit Construct(msg.sender, config);

        iEvaluableHash = config.evaluable.hash();
    }

    modifier onlyValidEvaluable(EvaluableV3 calldata evaluable) {
        if (evaluable.hash() != iEvaluableHash) {
            revert WrongEvaluable();
        }
        if (evaluable.bytecode.length > 0) {
            (uint256[] memory stack, uint256[] memory kvs) = evaluable.interpreter.eval3(
                evaluable.store,
                LibNamespace.qualifyNamespace(DEFAULT_STATE_NAMESPACE, address(this)),
                evaluable.bytecode,
                BEFORE_ARB_SOURCE_INDEX,
                LibContext.build(new uint256[][](0), new SignedContextV1[](0)),
                new uint256[](0)
            );
            // We don't care about the stack.
            (stack);
            // Persist any state changes from the expression.
            if (kvs.length > 0) {
                evaluable.store.set(DEFAULT_STATE_NAMESPACE, kvs);
            }
        }
        _;
    }
}
