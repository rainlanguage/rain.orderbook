// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity ^0.8.19;

import {
    CONTEXT_BASE_ROWS,
    CONTEXT_BASE_ROW_SENDER,
    CONTEXT_BASE_ROW_CALLING_CONTRACT,
    CONTEXT_BASE_COLUMN
} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {EvaluableV3, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {SourceIndexV2, StateNamespace} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";

/// @dev Orderbook context is actually fairly complex. The calling context column
/// is populated before calculate order, but the remaining columns are only
/// available to handle IO as they depend on the full evaluation of calculuate
/// order, and cross referencing against the same from the counterparty, as well
/// as accounting limits such as current vault balances, etc.
/// The token address and decimals for vault inputs and outputs IS available to
/// the calculate order entrypoint, but not the final vault balances/diff.
uint256 constant CALLING_CONTEXT_COLUMNS = 4;

uint256 constant CONTEXT_COLUMNS = CALLING_CONTEXT_COLUMNS + 1;

/// @dev Contextual data available to both calculate order and handle IO. The
/// order hash, order owner and order counterparty. IMPORTANT NOTE that the
/// typical base context of an order with the caller will often be an unrelated
/// clearer of the order rather than the owner or counterparty.
uint256 constant CONTEXT_CALLING_CONTEXT_COLUMN = 1;
uint256 constant CONTEXT_CALLING_CONTEXT_ROWS = 3;

uint256 constant CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH = 0;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER = 1;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY = 2;

uint256 constant CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN = 0;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID = 1;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BALANCE = 2;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_AMOUNT = 3;

uint256 constant CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN = 0;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID = 1;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BALANCE = 2;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_AMOUNT = 3;
uint256 constant CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT = 4;

/// @dev Calculations column contains the DECIMAL RESCALED calculations but
/// otherwise provided as-is according to calculate order entrypoint
uint256 constant CONTEXT_CALCULATIONS_COLUMN = 2;
uint256 constant CONTEXT_CALCULATIONS_ROWS = 2;

uint256 constant CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT = 0;
uint256 constant CONTEXT_CALCULATIONS_ROW_IO_RATIO = 1;

/// @dev Vault inputs are the literal token amounts and vault balances before and
/// after for the input token from the perspective of the order. MAY be
/// significantly different to the calculated amount due to insufficient vault
/// balances from either the owner or counterparty, etc.
uint256 constant CONTEXT_VAULT_INPUTS_COLUMN = 3;
/// @dev Vault outputs are the same as vault inputs but for the output token from
/// the perspective of the order.
uint256 constant CONTEXT_VAULT_OUTPUTS_COLUMN = 4;

/// @dev Row of the token address for vault inputs and outputs columns.
uint256 constant CONTEXT_VAULT_IO_TOKEN = 0;
/// @dev Row of the token decimals for vault inputs and outputs columns.
uint256 constant CONTEXT_VAULT_IO_TOKEN_DECIMALS = 1;
/// @dev Row of the vault ID for vault inputs and outputs columns.
uint256 constant CONTEXT_VAULT_IO_VAULT_ID = 2;
/// @dev Row of the vault balance before the order was cleared for vault inputs
/// and outputs columns.
uint256 constant CONTEXT_VAULT_IO_BALANCE_BEFORE = 3;
/// @dev Row of the vault balance difference after the order was cleared for
/// vault inputs and outputs columns. The diff is ALWAYS POSITIVE as it is a
/// `uint256` so it must be added to input balances and subtraced from output
/// balances.
uint256 constant CONTEXT_VAULT_IO_BALANCE_DIFF = 4;
/// @dev Length of a vault IO column.
uint256 constant CONTEXT_VAULT_IO_ROWS = 5;

uint256 constant CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN = 5;
uint256 constant CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS = 1;
uint256 constant CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW = 0;

uint256 constant CONTEXT_SIGNED_CONTEXT_START_COLUMN = 6;
uint256 constant CONTEXT_SIGNED_CONTEXT_START_ROWS = 1;
uint256 constant CONTEXT_SIGNED_CONTEXT_START_ROW = 0;

library LibOrderBook {
    function doPost(uint256[][] memory context, TaskV1[] memory post) internal {
        StateNamespace namespace = StateNamespace.wrap(uint256(uint160(msg.sender)));
        TaskV1 memory task;
        for (uint256 i = 0; i < post.length; ++i) {
            task = post[i];
            if (task.evaluable.bytecode.length > 0) {
                (uint256[] memory stack, uint256[] memory writes) = task.evaluable.interpreter.eval3(
                    task.evaluable.store,
                    LibNamespace.qualifyNamespace(namespace, address(this)),
                    task.evaluable.bytecode,
                    SourceIndexV2.wrap(0),
                    LibContext.build(context, task.signedContext),
                    new uint256[](0)
                );
                (stack);
                if (writes.length > 0) {
                    task.evaluable.store.set(namespace, writes);
                }
            }
        }
    }
}
