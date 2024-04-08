// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {EvaluableV3} from "rain.interpreter.interface/interface/unstable/IInterpreterCallerV3.sol";

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
/// @param evaluableConfig The `EvaluableConfigV3` to use as a pre-hook for each
/// arb.
/// @param implementationData The constructor data for the specific
/// implementation of the arb contract.
struct OrderBookV4ArbConfigV1 {
    address orderBook;
    EvaluableConfigV3 evaluableConfig;
    bytes implementationData;
}
