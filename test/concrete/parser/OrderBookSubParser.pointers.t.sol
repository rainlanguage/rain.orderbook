// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookSubParser, AuthoringMetaV2} from "src/lib/LibOrderBookSubParser.sol";
import {
    SUB_PARSER_PARSE_META,
    SUB_PARSER_WORD_PARSERS,
    SUB_PARSER_OPERAND_HANDLERS,
    OrderBookSubParser
} from "src/concrete/parser/OrderBookSubParser.sol";
import {LibParseMeta} from "rain.interpreter/lib/parse/LibParseMeta.sol";

contract OrderBookSubParserPointersTest is Test {
    function testSubParserParseMeta() external {
        bytes memory authoringMetaBytes = LibOrderBookSubParser.authoringMetaV2();
        AuthoringMetaV2[] memory authoringMeta = abi.decode(authoringMetaBytes, (AuthoringMetaV2[]));
        bytes memory expected = LibParseMeta.buildParseMetaV2(authoringMeta, 2);
        bytes memory actual = SUB_PARSER_PARSE_META;
        assertEq(actual, expected);
    }

    function testSubParserFunctionPointers() external {
        OrderBookSubParser extern = new OrderBookSubParser();
        bytes memory expected = extern.buildSubParserWordParsers();
        bytes memory actual = SUB_PARSER_WORD_PARSERS;
        assertEq(actual, expected);
    }

    function testSubParserOperandParsers() external {
        OrderBookSubParser extern = new OrderBookSubParser();
        bytes memory expected = extern.buildSubParserOperandHandlers();
        bytes memory actual = SUB_PARSER_OPERAND_HANDLERS;
        assertEq(actual, expected);
    }

    function testWordOperandLengthEquivalence() external {
        assertEq(SUB_PARSER_WORD_PARSERS.length, SUB_PARSER_OPERAND_HANDLERS.length);
        assertEq(
            SUB_PARSER_PARSE_META.length,
            // 4 bytes per word + 32 byte expansion + 1 byte seed + 1 byte depth
            (SUB_PARSER_WORD_PARSERS.length * 2) + 32 + 1 + 1
        );
    }
}
