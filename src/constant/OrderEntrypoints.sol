// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {SourceIndexV2} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";

/// @dev Entrypoint to a calculate the amount and ratio of an order.
SourceIndexV2 constant CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0);
/// @dev Entrypoint to handle the final internal vault movements resulting from
/// matching multiple calculated orders.
SourceIndexV2 constant HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1);

/// @dev Minimum outputs for calculate order are the amount and ratio.
uint256 constant CALCULATE_ORDER_MIN_OUTPUTS = 2;
/// @dev Maximum outputs for calculate order are the amount and ratio.
uint16 constant CALCULATE_ORDER_MAX_OUTPUTS = 2;

/// @dev Handle IO has no outputs as it only responds to vault movements.
uint256 constant HANDLE_IO_MIN_OUTPUTS = 0;
/// @dev Handle IO has no outputs as it only responds to vault movements.
uint16 constant HANDLE_IO_MAX_OUTPUTS = 0;
