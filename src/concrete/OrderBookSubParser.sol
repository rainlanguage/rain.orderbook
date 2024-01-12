// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {BaseRainterpreterSubParserNPE2} from "rain.interpreter/abstract/BaseRainterpreterSubParserNPE2.sol";

bytes constant SUB_PARSER_PARSE_META = hex"";
bytes constant SUB_PARSER_WORD_PARSERS = hex"";
bytes constant SUB_PARSER_OPERAND_HANDLERS = hex"";

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

}
