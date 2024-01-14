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
    CONTEXT_BASE_COLUMN
} from "../ob/OrderBook.sol";

bytes constant SUB_PARSER_PARSE_META =
    hex"010000000000000200000000000000000000040000000000000000000000000000000109ac3000d3b4e8";
bytes constant SUB_PARSER_WORD_PARSERS = hex"0540055f";
bytes constant SUB_PARSER_OPERAND_HANDLERS = hex"06010601";

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
        unchecked {
            function(uint256[] memory) internal pure returns (Operand)[][] memory handlers =
                new function(uint256[] memory) internal pure returns (Operand)[][](CONTEXT_COLUMNS);

            function(uint256[] memory) internal pure returns (Operand)[] memory contextBaseHandlers =
                new function(uint256[] memory) internal pure returns (Operand)[](CONTEXT_BASE_ROWS);
            contextBaseHandlers[CONTEXT_BASE_ROW_SENDER] = LibParseOperand.handleOperandDisallowed;
            contextBaseHandlers[CONTEXT_BASE_ROW_CALLING_CONTRACT] = LibParseOperand.handleOperandDisallowed;

            handlers[CONTEXT_BASE_COLUMN] = contextBaseHandlers;

            uint256[][] memory handlersUint256;
            assembly ("memory-safe") {
                handlersUint256 := handlers
            }

            return LibConvert.unsafeTo16BitBytes(handlersUint256.flatten());
        }
    }

    function buildSubParserWordParsers() external pure returns (bytes memory) {
        unchecked {
            function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)
                lengthPointer;
            uint256 length = SUB_PARSER_WORD_PARSERS_LENGTH;
            assembly ("memory-safe") {
                lengthPointer := length
            }
            function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[SUB_PARSER_WORD_PARSERS_LENGTH
                + 1] memory pointersFixed = [
                    lengthPointer,
                    LibExternOpContextSenderNPE2.subParser,
                    LibExternOpContextCallingContractNPE2.subParser
                ];
            uint256[] memory pointersDynamic;
            assembly {
                pointersDynamic := pointersFixed
            }
            // Sanity check that the dynamic length is correct. Should be an
            // unreachable error.
            if (pointersDynamic.length != length) {
                revert BadDynamicLength(pointersDynamic.length, length);
            }
            return LibConvert.unsafeTo16BitBytes(pointersDynamic);
        }
    }
}
