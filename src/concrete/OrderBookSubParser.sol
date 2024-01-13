// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {
    LibParseOperand,
    BaseRainterpreterSubParserNPE2,
    Operand
} from "rain.interpreter/abstract/BaseRainterpreterSubParserNPE2.sol";
import {LibOrderBookSubParser} from "../lib/LibOrderBookSubParser.sol";
import {LibConvert} from "rain.lib.typecast/LibConvert.sol";
import {BadDynamicLength} from "rain.interpreter/error/ErrOpList.sol";
import {LibExternOpContextSenderNPE2} from "rain.interpreter/lib/extern/reference/op/LibExternOpContextSenderNPE2.sol";

bytes constant SUB_PARSER_PARSE_META = hex"0100000000000002000000000000000000000000000000000000000000000000000000d3b4e8";
bytes constant SUB_PARSER_WORD_PARSERS = hex"044b";
bytes constant SUB_PARSER_OPERAND_HANDLERS = hex"04fb";
uint256 constant SUB_PARSER_WORD_PARSERS_LENGTH = 1;

contract OrderBookSubParser is BaseRainterpreterSubParserNPE2 {
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
            function(uint256[] memory) internal pure returns (Operand) lengthPointer;
            uint256 length = SUB_PARSER_WORD_PARSERS_LENGTH;
            assembly ("memory-safe") {
                lengthPointer := length
            }
            function(uint256[] memory) internal pure returns (Operand)[SUB_PARSER_WORD_PARSERS_LENGTH + 1] memory
                handlersFixed = [
                    lengthPointer,
                    // order clearer
                    LibParseOperand.handleOperandDisallowed
                ];
            uint256[] memory handlersDynamic;
            assembly {
                handlersDynamic := handlersFixed
            }
            // Sanity check that the dynamic length is correct. Should be an
            // unreachable error.
            if (handlersDynamic.length != length) {
                revert BadDynamicLength(handlersDynamic.length, length);
            }
            return LibConvert.unsafeTo16BitBytes(handlersDynamic);
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
                + 1] memory pointersFixed = [lengthPointer, LibExternOpContextSenderNPE2.subParser];
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
