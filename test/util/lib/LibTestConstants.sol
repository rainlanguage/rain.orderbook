// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

/// @dev Mocks need to be etched with some bytecode or they cannot even be
/// called. This is because Solidity first checks the bytecode size before
/// calling, so it never even gets to the point that mocking logic can intercept
/// the call. We want all non-mocked calls to revert, so all mocks should be
/// etched with a revert opcode.
bytes constant REVERTING_MOCK_BYTECODE = hex"FD";
