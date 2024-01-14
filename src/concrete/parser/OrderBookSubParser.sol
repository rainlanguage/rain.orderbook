// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {
    LibParseOperand,
    BaseRainterpreterSubParserNPE2,
    Operand
} from "rain.interpreter/abstract/BaseRainterpreterSubParserNPE2.sol";
import {LibConvert} from "rain.lib.typecast/LibConvert.sol";
import {BadDynamicLength} from "rain.interpreter/error/ErrOpList.sol";
import {LibExternOpContextSenderNPE2} from "rain.interpreter/lib/extern/reference/op/LibExternOpContextSenderNPE2.sol";
import {LibExternOpContextCallingContractNPE2} from
    "rain.interpreter/lib/extern/reference/op/LibExternOpContextCallingContractNPE2.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";

import {LibOrderBookSubParser, SUB_PARSER_WORD_PARSERS_LENGTH} from "../../lib/LibOrderBookSubParser.sol";
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
    CONTEXT_CALLING_CONTEXT_ROWS
} from "../../lib/LibOrderBook.sol";

bytes constant SUB_PARSER_PARSE_META =
    hex"010000000000000200000000000000000000040000000000000000000000000000000109ac3000d3b4e8";
bytes constant SUB_PARSER_WORD_PARSERS = hex"06030622";
bytes constant SUB_PARSER_OPERAND_HANDLERS = hex"07580758";

contract OrderBookSubParser is BaseRainterpreterSubParserNPE2 {
    using LibUint256Matrix for uint256[][];

    function subParserParseMeta() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_PARSE_META;
    }

    function subParserWordParsers() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_WORD_PARSERS;
    }

    function subParserOperandHandlers() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_OPERAND_HANDLERS;
    }

    function buildSubParserOperandHandlers() external pure returns (bytes memory) {
        function(uint256[] memory) internal pure returns (Operand)[][] memory handlers =
            new function(uint256[] memory) internal pure returns (Operand)[][](CONTEXT_COLUMNS);

        function(uint256[] memory) internal pure returns (Operand)[] memory contextBaseHandlers =
            new function(uint256[] memory) internal pure returns (Operand)[](CONTEXT_BASE_ROWS);
        contextBaseHandlers[CONTEXT_BASE_ROW_SENDER] = LibParseOperand.handleOperandDisallowed;
        contextBaseHandlers[CONTEXT_BASE_ROW_CALLING_CONTRACT] = LibParseOperand.handleOperandDisallowed;

        function(uint256[] memory) internal pure returns (Operand)[] memory contextCallingContextHandlers =
            new function(uint256[] memory) internal pure returns (Operand)[](CONTEXT_BASE_ROWS);
        contextCallingContextHandlers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH] = LibParseOperand.handleOperandDisallowed;
        contextCallingContextHandlers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER] = LibParseOperand.handleOperandDisallowed;
        contextCallingContextHandlers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY] =
            LibParseOperand.handleOperandDisallowed;

        function(uint256[] memory) internal pure returns (Operand)[] memory contextCalculationsHandlers =
            new function(uint256[] memory) internal pure returns (Operand)[](CONTEXT_CALCULATIONS_ROWS);
        contextCalculationsHandlers[CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT] = LibParseOperand.handleOperandDisallowed;
        contextCalculationsHandlers[CONTEXT_CALCULATIONS_ROW_IO_RATIO] = LibParseOperand.handleOperandDisallowed;

        function(uint256[] memory) internal pure returns (Operand)[] memory contextVaultInputsHandlers =
            new function(uint256[] memory) internal pure returns (Operand)[](CONTEXT_VAULT_IO_ROWS);
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_TOKEN] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_VAULT_ID] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibParseOperand.handleOperandDisallowed;
        contextVaultInputsHandlers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibParseOperand.handleOperandDisallowed;

        function(uint256[] memory) internal pure returns (Operand)[] memory contextVaultOutputsHandlers =
            new function(uint256[] memory) internal pure returns (Operand)[](CONTEXT_VAULT_IO_ROWS);
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_TOKEN] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_VAULT_ID] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibParseOperand.handleOperandDisallowed;
        contextVaultOutputsHandlers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibParseOperand.handleOperandDisallowed;

        handlers[CONTEXT_BASE_COLUMN] = contextBaseHandlers;
        handlers[CONTEXT_BASE_COLUMN] = contextCallingContextHandlers;
        handlers[CONTEXT_CALCULATIONS_COLUMN] = contextCalculationsHandlers;
        handlers[CONTEXT_VAULT_INPUTS_COLUMN] = contextVaultInputsHandlers;
        handlers[CONTEXT_VAULT_OUTPUTS_COLUMN] = contextVaultOutputsHandlers;

        uint256[][] memory handlersUint256;
        assembly ("memory-safe") {
            handlersUint256 := handlers
        }

        return LibConvert.unsafeTo16BitBytes(handlersUint256.flatten());
    }

    function buildSubParserWordParsers() external pure returns (bytes memory) {
        function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[][] memory
            parsers = new function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[][](
                CONTEXT_COLUMNS
            );

        function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[] memory
            contextBaseParsers = new function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[](
                CONTEXT_BASE_ROWS
            );
        contextBaseParsers[CONTEXT_BASE_ROW_SENDER] = LibOrderBookSubParser.subParserSender;
        contextBaseParsers[CONTEXT_BASE_ROW_CALLING_CONTRACT] = LibOrderBookSubParser.subParserCallingContract;

        function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[] memory
            contextCallingContextParsers = new function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[](
                CONTEXT_CALLING_CONTEXT_ROWS
            );
        contextCallingContextParsers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH] = LibOrderBookSubParser.subParserOrderHash;
        contextCallingContextParsers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER] =
            LibOrderBookSubParser.subParserOrderOwner;
        contextCallingContextParsers[CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY] =
            LibOrderBookSubParser.subParserOrderCounterparty;

        function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[] memory
            contextCalculationsParsers = new function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[](
                CONTEXT_CALCULATIONS_ROWS
            );
        contextCalculationsParsers[CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT] = LibOrderBookSubParser.subParserMaxOutput;
        contextCalculationsParsers[CONTEXT_CALCULATIONS_ROW_IO_RATIO] = LibOrderBookSubParser.subParserIORatio;

        function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[] memory
            contextVaultInputsParsers = new function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[](
                CONTEXT_VAULT_IO_ROWS
            );
        contextVaultInputsParsers[CONTEXT_VAULT_IO_TOKEN] = LibOrderBookSubParser.subParserInputToken;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibOrderBookSubParser.subParserInputTokenDecimals;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_VAULT_ID] = LibOrderBookSubParser.subParserInputVaultId;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibOrderBookSubParser.subParserInputBalanceBefore;
        contextVaultInputsParsers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibOrderBookSubParser.subParserInputBalanceDiff;

        function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[] memory
            contextVaultOutputsParsers = new function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[](
                CONTEXT_VAULT_IO_ROWS
            );
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_TOKEN] = LibOrderBookSubParser.subParserOutputToken;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = LibOrderBookSubParser.subParserOutputTokenDecimals;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_VAULT_ID] = LibOrderBookSubParser.subParserOutputVaultId;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_BALANCE_BEFORE] = LibOrderBookSubParser.subParserOutputBalanceBefore;
        contextVaultOutputsParsers[CONTEXT_VAULT_IO_BALANCE_DIFF] = LibOrderBookSubParser.subParserOutputBalanceDiff;

        parsers[CONTEXT_BASE_COLUMN] = contextBaseParsers;
        parsers[CONTEXT_BASE_COLUMN] = contextCallingContextParsers;
        parsers[CONTEXT_CALCULATIONS_COLUMN] = contextCalculationsParsers;
        parsers[CONTEXT_VAULT_INPUTS_COLUMN] = contextVaultInputsParsers;
        parsers[CONTEXT_VAULT_OUTPUTS_COLUMN] = contextVaultOutputsParsers;

        uint256[][] memory parsersUint256;
        assembly ("memory-safe") {
            parsersUint256 := parsers
        }

        return LibConvert.unsafeTo16BitBytes(parsersUint256.flatten());
    }
}
