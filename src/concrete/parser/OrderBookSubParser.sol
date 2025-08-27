// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    LibParseOperand,
    BaseRainterpreterSubParserNPE2,
    OperandV2,
    IParserToolingV1
} from "rain.interpreter/abstract/BaseRainterpreterSubParserNPE2.sol";
import {LibConvert} from "rain.lib.typecast/LibConvert.sol";
import {BadDynamicLength} from "rain.interpreter/error/ErrOpList.sol";
import {LibExternOpContextSenderNPE2} from "rain.interpreter/lib/extern/reference/op/LibExternOpContextSenderNPE2.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";

import {
    LibOrderBookSubParser,
    SUB_PARSER_WORD_PARSERS_LENGTH,
    DEPOSIT_WORD_VAULT_ID,
    DEPOSIT_WORD_TOKEN,
    DEPOSIT_WORD_DEPOSITOR,
    DEPOSIT_WORD_VAULT_BEFORE,
    DEPOSIT_WORD_VAULT_AFTER,
    DEPOSIT_WORDS_LENGTH,
    WITHDRAW_WORD_WITHDRAWER,
    WITHDRAW_WORD_TOKEN,
    WITHDRAW_WORD_VAULT_ID,
    WITHDRAW_WORD_VAULT_BEFORE,
    WITHDRAW_WORD_VAULT_AFTER,
    WITHDRAW_WORD_TARGET_AMOUNT,
    WITHDRAW_WORDS_LENGTH
} from "../../lib/LibOrderBookSubParser.sol";
import {
    CONTEXT_COLUMNS,
    CONTEXT_BASE_ROWS,
    CONTEXT_BASE_ROW_SENDER,
    CONTEXT_BASE_ROW_CALLING_CONTRACT,
    CONTEXT_BASE_COLUMN,
    CONTEXT_VAULT_OUTPUTS_COLUMN,
    CONTEXT_VAULT_INPUTS_COLUMN,
    CONTEXT_CALCULATIONS_COLUMN,
    CONTEXT_VAULT_IO_BALANCE_DIFF,
    CONTEXT_VAULT_IO_BALANCE_BEFORE,
    CONTEXT_VAULT_IO_VAULT_ID,
    CONTEXT_VAULT_IO_TOKEN_DECIMALS,
    CONTEXT_VAULT_IO_TOKEN,
    CONTEXT_VAULT_IO_ROWS,
    CONTEXT_CALCULATIONS_ROW_IO_RATIO,
    CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT,
    CONTEXT_CALCULATIONS_ROWS,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH,
    CONTEXT_CALLING_CONTEXT_ROWS,
    CONTEXT_CALLING_CONTEXT_COLUMN,
    CONTEXT_SIGNED_CONTEXT_START_COLUMN,
    CONTEXT_SIGNED_CONTEXT_START_ROW,
    CONTEXT_SIGNED_CONTEXT_START_ROWS,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS
} from "../../lib/LibOrderBook.sol";
import {
    DESCRIBED_BY_META_HASH,
    PARSE_META as SUB_PARSER_PARSE_META,
    SUB_PARSER_WORD_PARSERS,
    OPERAND_HANDLER_FUNCTION_POINTERS as SUB_PARSER_OPERAND_HANDLERS
} from "../../generated/OrderBookSubParser.pointers.sol";
import {IDescribedByMetaV1} from "rain.metadata/interface/IDescribedByMetaV1.sol";

contract OrderBookSubParser is BaseRainterpreterSubParserNPE2 {
    using LibUint256Matrix for uint256[][];

    /// @inheritdoc IDescribedByMetaV1
    function describedByMetaV1() external pure returns (bytes32) {
        return DESCRIBED_BY_META_HASH;
    }

    /// @inheritdoc BaseRainterpreterSubParserNPE2
    function subParserParseMeta() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_PARSE_META;
    }

    /// @inheritdoc BaseRainterpreterSubParserNPE2
    function subParserWordParsers() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_WORD_PARSERS;
    }

    /// @inheritdoc BaseRainterpreterSubParserNPE2
    function subParserOperandHandlers() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_OPERAND_HANDLERS;
    }

    /// @inheritdoc IParserToolingV1
    function buildLiteralParserFunctionPointers() external pure returns (bytes memory) {
        return "";
    }

    /// @inheritdoc IParserToolingV1
    function buildOperandHandlerFunctionPointers() external pure returns (bytes memory) {
        // Add 2 columns for signers and signed context start.
        // Add 1 for deposit context
        // Add 1 for withdraw context
        function(bytes32[] memory) internal pure returns (OperandV2)[][] memory handlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[][](CONTEXT_COLUMNS + 2 + 1 + 1);

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextBaseHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_BASE_ROWS);
        contextBaseHandlers[CONTEXT_BASE_ROW_SENDER] = LibParseOperand.handleOperandDisallowed;
        contextBaseHandlers[CONTEXT_BASE_ROW_CALLING_CONTRACT] = LibParseOperand.handleOperandDisallowed;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextCallingContextHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_CALLING_CONTEXT_ROWS);
        contextCallingContextHandlers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH] = LibParseOperand.handleOperandDisallowed;
        contextCallingContextHandlers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER] = LibParseOperand.handleOperandDisallowed;
        contextCallingContextHandlers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY] =
            LibParseOperand.handleOperandDisallowed;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextCalculationsHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_CALCULATIONS_ROWS);
        contextCalculationsHandlers[CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT] = LibParseOperand.handleOperandDisallowed;
        contextCalculationsHandlers[CONTEXT_CALCULATIONS_ROW_IO_RATIO] = LibParseOperand.handleOperandDisallowed;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextVaultInputsHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_VAULT_IO_ROWS);
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_TOKEN] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_VAULT_ID] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibParseOperand.handleOperandDisallowed;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextVaultOutputsHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_VAULT_IO_ROWS);
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_TOKEN] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_VAULT_ID] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibParseOperand.handleOperandDisallowed;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextSignersHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS);
        contextSignersHandlers[CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW] = LibParseOperand.handleOperandSingleFullNoDefault;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextSignedContextHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](CONTEXT_SIGNED_CONTEXT_START_ROWS);
        contextSignedContextHandlers[CONTEXT_SIGNED_CONTEXT_START_ROW] =
            LibParseOperand.handleOperandDoublePerByteNoDefault;

        handlers[CONTEXT_BASE_COLUMN] = contextBaseHandlers;
        handlers[CONTEXT_CALLING_CONTEXT_COLUMN] = contextCallingContextHandlers;
        handlers[CONTEXT_CALCULATIONS_COLUMN] = contextCalculationsHandlers;
        handlers[CONTEXT_VAULT_INPUTS_COLUMN] = contextVaultInputsHandlers;
        handlers[CONTEXT_VAULT_OUTPUTS_COLUMN] = contextVaultOutputsHandlers;
        handlers[CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN] = contextSignersHandlers;
        handlers[CONTEXT_SIGNED_CONTEXT_START_COLUMN] = contextSignedContextHandlers;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextDepositContextHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](DEPOSIT_WORDS_LENGTH);
        contextDepositContextHandlers[DEPOSIT_WORD_DEPOSITOR] = LibParseOperand.handleOperandDisallowed;
        contextDepositContextHandlers[DEPOSIT_WORD_TOKEN] = LibParseOperand.handleOperandDisallowed;
        contextDepositContextHandlers[DEPOSIT_WORD_VAULT_ID] = LibParseOperand.handleOperandDisallowed;
        contextDepositContextHandlers[DEPOSIT_WORD_VAULT_BEFORE] = LibParseOperand.handleOperandDisallowed;
        contextDepositContextHandlers[DEPOSIT_WORD_VAULT_AFTER] = LibParseOperand.handleOperandDisallowed;

        handlers[CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1] = contextDepositContextHandlers;

        function(bytes32[] memory) internal pure returns (OperandV2)[] memory contextWithdrawContextHandlers =
            new function(bytes32[] memory) internal pure returns (OperandV2)[](WITHDRAW_WORDS_LENGTH);
        contextWithdrawContextHandlers[WITHDRAW_WORD_WITHDRAWER] = LibParseOperand.handleOperandDisallowed;
        contextWithdrawContextHandlers[WITHDRAW_WORD_TOKEN] = LibParseOperand.handleOperandDisallowed;
        contextWithdrawContextHandlers[WITHDRAW_WORD_VAULT_ID] = LibParseOperand.handleOperandDisallowed;
        contextWithdrawContextHandlers[WITHDRAW_WORD_VAULT_BEFORE] = LibParseOperand.handleOperandDisallowed;
        contextWithdrawContextHandlers[WITHDRAW_WORD_VAULT_AFTER] = LibParseOperand.handleOperandDisallowed;
        contextWithdrawContextHandlers[WITHDRAW_WORD_TARGET_AMOUNT] = LibParseOperand.handleOperandDisallowed;

        handlers[CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2] = contextWithdrawContextHandlers;

        uint256[][] memory handlersUint256;
        assembly ("memory-safe") {
            handlersUint256 := handlers
        }

        return LibConvert.unsafeTo16BitBytes(handlersUint256.flatten());
    }

    function buildSubParserWordParsers() external pure returns (bytes memory) {
        // Add 2 columns for signers and signed context start.
        // Add 1 for deposit context
        // Add 1 for withdraw context
        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[][] memory
            parsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[][](
                CONTEXT_COLUMNS + 2 + 1 + 1
            );

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextBaseParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_BASE_ROWS
            );
        contextBaseParsers[CONTEXT_BASE_ROW_SENDER] = LibOrderBookSubParser.subParserSender;
        contextBaseParsers[CONTEXT_BASE_ROW_CALLING_CONTRACT] = LibOrderBookSubParser.subParserCallingContract;

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextCallingContextParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_CALLING_CONTEXT_ROWS
            );
        contextCallingContextParsers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH] = LibOrderBookSubParser.subParserOrderHash;
        contextCallingContextParsers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER] =
            LibOrderBookSubParser.subParserOrderOwner;
        contextCallingContextParsers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY] =
            LibOrderBookSubParser.subParserOrderCounterparty;

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextCalculationsParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_CALCULATIONS_ROWS
            );
        contextCalculationsParsers[CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT] = LibOrderBookSubParser.subParserMaxOutput;
        contextCalculationsParsers[CONTEXT_CALCULATIONS_ROW_IO_RATIO] = LibOrderBookSubParser.subParserIORatio;

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextVaultInputsParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_VAULT_IO_ROWS
            );
        contextVaultInputsParsers[CONTEXT_VAULT_IO_TOKEN] = LibOrderBookSubParser.subParserInputToken;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibOrderBookSubParser.subParserInputTokenDecimals;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_VAULT_ID] = LibOrderBookSubParser.subParserInputVaultId;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibOrderBookSubParser.subParserInputBalanceBefore;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibOrderBookSubParser.subParserInputBalanceDiff;

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextVaultOutputsParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_VAULT_IO_ROWS
            );
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_TOKEN] = LibOrderBookSubParser.subParserOutputToken;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibOrderBookSubParser.subParserOutputTokenDecimals;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_VAULT_ID] = LibOrderBookSubParser.subParserOutputVaultId;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibOrderBookSubParser.subParserOutputBalanceBefore;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibOrderBookSubParser.subParserOutputBalanceDiff;

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextSignersParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS
            );
        contextSignersParsers[CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW] = LibOrderBookSubParser.subParserSigners;

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            contextSignedContextParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                CONTEXT_SIGNED_CONTEXT_START_ROWS
            );
        contextSignedContextParsers[CONTEXT_SIGNED_CONTEXT_START_ROW] = LibOrderBookSubParser.subParserSignedContext;

        parsers[CONTEXT_BASE_COLUMN] = contextBaseParsers;
        parsers[CONTEXT_CALLING_CONTEXT_COLUMN] = contextCallingContextParsers;
        parsers[CONTEXT_CALCULATIONS_COLUMN] = contextCalculationsParsers;
        parsers[CONTEXT_VAULT_INPUTS_COLUMN] = contextVaultInputsParsers;
        parsers[CONTEXT_VAULT_OUTPUTS_COLUMN] = contextVaultOutputsParsers;
        parsers[CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN] = contextSignersParsers;
        parsers[CONTEXT_SIGNED_CONTEXT_START_COLUMN] = contextSignedContextParsers;

        // Deposits

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            depositParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                DEPOSIT_WORDS_LENGTH
            );

        depositParsers[DEPOSIT_WORD_DEPOSITOR] = LibOrderBookSubParser.subParserSender;
        depositParsers[DEPOSIT_WORD_TOKEN] = LibOrderBookSubParser.subParserDepositToken;
        depositParsers[DEPOSIT_WORD_VAULT_ID] = LibOrderBookSubParser.subParserDepositVaultId;
        depositParsers[DEPOSIT_WORD_VAULT_BEFORE] = LibOrderBookSubParser.subParserDepositVaultBalanceBefore;
        depositParsers[DEPOSIT_WORD_VAULT_AFTER] = LibOrderBookSubParser.subParserDepositVaultBalanceAfter;

        parsers[CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1] = depositParsers;

        // Withdraws

        function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[] memory
            withdrawParsers = new function(uint256, uint256, OperandV2) internal view returns (bool, bytes memory, bytes32[] memory)[](
                WITHDRAW_WORDS_LENGTH
            );

        withdrawParsers[WITHDRAW_WORD_WITHDRAWER] = LibOrderBookSubParser.subParserSender;
        withdrawParsers[WITHDRAW_WORD_TOKEN] = LibOrderBookSubParser.subParserWithdrawToken;
        withdrawParsers[WITHDRAW_WORD_VAULT_ID] = LibOrderBookSubParser.subParserWithdrawVaultId;
        withdrawParsers[WITHDRAW_WORD_VAULT_BEFORE] = LibOrderBookSubParser.subParserWithdrawVaultBalanceBefore;
        withdrawParsers[WITHDRAW_WORD_VAULT_AFTER] = LibOrderBookSubParser.subParserWithdrawVaultBalanceAfter;
        withdrawParsers[WITHDRAW_WORD_TARGET_AMOUNT] = LibOrderBookSubParser.subParserWithdrawTargetAmount;

        parsers[CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2] = withdrawParsers;

        uint256[][] memory parsersUint256;
        assembly ("memory-safe") {
            parsersUint256 := parsers
        }

        return LibConvert.unsafeTo16BitBytes(parsersUint256.flatten());
    }
}
