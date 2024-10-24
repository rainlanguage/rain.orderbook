// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {
    WORD_ORDERBOOK,
    WORD_ORDER_CLEARER,
    WORD_ORDER_HASH,
    WORD_ORDER_OWNER,
    WORD_ORDER_COUNTERPARTY,
    CONTEXT_COLUMNS,
    CONTEXT_BASE_ROWS,
    CONTEXT_BASE_ROW_SENDER,
    CONTEXT_BASE_ROW_CALLING_CONTRACT,
    CONTEXT_CALLING_CONTEXT_ROWS,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY,
    CONTEXT_BASE_COLUMN,
    CONTEXT_CALLING_CONTEXT_COLUMN,
    CONTEXT_CALCULATIONS_COLUMN,
    CONTEXT_CALCULATIONS_ROWS,
    CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT,
    CONTEXT_CALCULATIONS_ROW_IO_RATIO,
    WORD_CALCULATED_MAX_OUTPUT,
    WORD_CALCULATED_IO_RATIO,
    CONTEXT_VAULT_IO_ROWS,
    CONTEXT_VAULT_INPUTS_COLUMN,
    CONTEXT_VAULT_OUTPUTS_COLUMN,
    CONTEXT_VAULT_IO_TOKEN,
    CONTEXT_VAULT_IO_TOKEN_DECIMALS,
    CONTEXT_VAULT_IO_VAULT_ID,
    CONTEXT_VAULT_IO_BALANCE_BEFORE,
    CONTEXT_VAULT_IO_BALANCE_DIFF,
    WORD_INPUT_TOKEN,
    WORD_INPUT_TOKEN_DECIMALS,
    WORD_INPUT_VAULT_ID,
    WORD_INPUT_VAULT_BALANCE_BEFORE,
    WORD_INPUT_VAULT_BALANCE_INCREASE,
    WORD_OUTPUT_TOKEN,
    WORD_OUTPUT_TOKEN_DECIMALS,
    WORD_OUTPUT_VAULT_ID,
    WORD_OUTPUT_VAULT_BALANCE_BEFORE,
    WORD_OUTPUT_VAULT_BALANCE_DECREASE,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN,
    CONTEXT_SIGNED_CONTEXT_START_COLUMN
} from "src/lib/LibOrderBookSubParser.sol";

library LibOrderBookSubParserContextFixture {
    function hashedNamesContext() internal pure returns (uint256[][] memory) {
        // Add 3 to account for the signers and 2x signed context columns.
        uint256[][] memory context = new uint256[][](CONTEXT_COLUMNS + 3);

        uint256[] memory contextBase = new uint256[](CONTEXT_BASE_ROWS);
        contextBase[CONTEXT_BASE_ROW_SENDER] = uint256(keccak256(WORD_ORDER_CLEARER));
        contextBase[CONTEXT_BASE_ROW_CALLING_CONTRACT] = uint256(keccak256(WORD_ORDERBOOK));

        uint256[] memory contextCallingContext = new uint256[](CONTEXT_CALLING_CONTEXT_ROWS);
        contextCallingContext[CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH] = uint256(keccak256(WORD_ORDER_HASH));
        contextCallingContext[CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER] = uint256(keccak256(WORD_ORDER_OWNER));
        contextCallingContext[CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY] =
            uint256(keccak256(WORD_ORDER_COUNTERPARTY));

        uint256[] memory contextCalculations = new uint256[](CONTEXT_CALCULATIONS_ROWS);
        contextCalculations[CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT] = uint256(keccak256(WORD_CALCULATED_MAX_OUTPUT));
        contextCalculations[CONTEXT_CALCULATIONS_ROW_IO_RATIO] = uint256(keccak256(WORD_CALCULATED_IO_RATIO));

        uint256[] memory contextVaultInputs = new uint256[](CONTEXT_VAULT_IO_ROWS);
        contextVaultInputs[CONTEXT_VAULT_IO_TOKEN] = uint256(keccak256(WORD_INPUT_TOKEN));
        contextVaultInputs[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = uint256(keccak256(WORD_INPUT_TOKEN_DECIMALS));
        contextVaultInputs[CONTEXT_VAULT_IO_VAULT_ID] = uint256(keccak256(WORD_INPUT_VAULT_ID));
        contextVaultInputs[CONTEXT_VAULT_IO_BALANCE_BEFORE] = uint256(keccak256(WORD_INPUT_VAULT_BALANCE_BEFORE));
        contextVaultInputs[CONTEXT_VAULT_IO_BALANCE_DIFF] = uint256(keccak256(WORD_INPUT_VAULT_BALANCE_INCREASE));

        uint256[] memory contextVaultOutputs = new uint256[](CONTEXT_VAULT_IO_ROWS);
        contextVaultOutputs[CONTEXT_VAULT_IO_TOKEN] = uint256(keccak256(WORD_OUTPUT_TOKEN));
        contextVaultOutputs[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = uint256(keccak256(WORD_OUTPUT_TOKEN_DECIMALS));
        contextVaultOutputs[CONTEXT_VAULT_IO_VAULT_ID] = uint256(keccak256(WORD_OUTPUT_VAULT_ID));
        contextVaultOutputs[CONTEXT_VAULT_IO_BALANCE_BEFORE] = uint256(keccak256(WORD_OUTPUT_VAULT_BALANCE_BEFORE));
        contextVaultOutputs[CONTEXT_VAULT_IO_BALANCE_DIFF] = uint256(keccak256(WORD_OUTPUT_VAULT_BALANCE_DECREASE));

        uint256[] memory contextSigners = new uint256[](2);
        contextSigners[0] = uint256(keccak256("signer-0"));
        contextSigners[1] = uint256(keccak256("signer-1"));

        uint256[] memory contextSignedContext0 = new uint256[](2);
        contextSignedContext0[0] = uint256(keccak256("signed-context-0-0"));
        contextSignedContext0[1] = uint256(keccak256("signed-context-0-1"));

        uint256[] memory contextSignedContext1 = new uint256[](2);
        contextSignedContext1[0] = uint256(keccak256("signed-context-1-0"));
        contextSignedContext1[1] = uint256(keccak256("signed-context-1-1"));

        context[CONTEXT_BASE_COLUMN] = contextBase;
        context[CONTEXT_CALLING_CONTEXT_COLUMN] = contextCallingContext;
        context[CONTEXT_CALCULATIONS_COLUMN] = contextCalculations;
        context[CONTEXT_VAULT_INPUTS_COLUMN] = contextVaultInputs;
        context[CONTEXT_VAULT_OUTPUTS_COLUMN] = contextVaultOutputs;
        context[CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN] = contextSigners;
        context[CONTEXT_SIGNED_CONTEXT_START_COLUMN] = contextSignedContext0;
        context[CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1] = contextSignedContext1;

        return context;
    }
}
