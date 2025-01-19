// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.18;

/// @dev Mocks need to be etched with some bytecode or they cannot even be
/// called. This is because Solidity first checks the bytecode size before
/// calling, so it never even gets to the point that mocking logic can intercept
/// the call. We want all non-mocked calls to revert, so all mocks should be
/// etched with a revert opcode.
bytes constant REVERTING_MOCK_BYTECODE = hex"FD";

/// @dev The address of the console.
/// This is a constant instead of just referring to `console` directly because
/// of a bug in foundry https://github.com/foundry-rs/foundry/issues/5311
address constant CONSOLE_ADDRESS = address(0x000000000000000000636F6e736F6c652e6c6f67);
