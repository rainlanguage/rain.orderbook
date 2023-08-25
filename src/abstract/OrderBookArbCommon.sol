// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

/// Thrown when the minimum output for the sender is not met after the arb.
/// @param minimum The minimum output expected by the sender.
/// @param actual The actual output that would be received by the sender.
error MinimumOutput(uint256 minimum, uint256 actual);

/// Thrown when calling functions while the contract is still initializing.
error Initializing();

/// Thrown when the stack is not empty after the access control dispatch.
error NonZeroBeforeArbStack();