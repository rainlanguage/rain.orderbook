// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookSubParser, AuthoringMetaV2} from "src/lib/LibOrderBookSubParser.sol";
import {
    SUB_PARSER_PARSE_META,
    SUB_PARSER_WORD_PARSERS,
    SUB_PARSER_OPERAND_HANDLERS,
    OrderBookSubParser
} from "src/concrete/OrderBookSubParser.sol";
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
}
