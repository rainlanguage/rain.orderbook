// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookSubParser, AuthoringMetaV2} from "src/lib/LibOrderBookSubParser.sol";
import {
    SUB_PARSER_PARSE_META,
    SUB_PARSER_WORD_PARSERS,
    SUB_PARSER_OPERAND_HANDLERS,
    OrderBookV6SubParser
} from "src/concrete/parser/OrderBookV6SubParser.sol";
import {LibGenParseMeta} from "rain.interpreter.interface/lib/codegen/LibGenParseMeta.sol";

contract OrderBookV6SubParserPointersTest is Test {
    function testSubParserParseMeta() external pure {
        bytes memory authoringMetaBytes = LibOrderBookSubParser.authoringMetaV2();
        AuthoringMetaV2[] memory authoringMeta = abi.decode(authoringMetaBytes, (AuthoringMetaV2[]));
        bytes memory expected = LibGenParseMeta.buildParseMetaV2(authoringMeta, 2);
        bytes memory actual = SUB_PARSER_PARSE_META;
        assertEq(actual, expected);
    }

    function testSubParserFunctionPointers() external {
        OrderBookV6SubParser extern = new OrderBookV6SubParser();
        bytes memory expected = extern.buildSubParserWordParsers();
        bytes memory actual = SUB_PARSER_WORD_PARSERS;
        assertEq(actual, expected);
    }

    function testSubParserOperandParsers() external {
        OrderBookV6SubParser extern = new OrderBookV6SubParser();
        bytes memory expected = extern.buildOperandHandlerFunctionPointers();
        bytes memory actual = SUB_PARSER_OPERAND_HANDLERS;
        assertEq(actual, expected);
    }

    function testWordOperandLengthEquivalence() external pure {
        assertEq(SUB_PARSER_WORD_PARSERS.length, SUB_PARSER_OPERAND_HANDLERS.length);
        assertEq(
            SUB_PARSER_PARSE_META.length,
            // 4 bytes per word + 32 byte expansion + 1 byte seed + 1 byte depth
            (SUB_PARSER_WORD_PARSERS.length * 2) + 32 + 1 + 1
        );
    }
}
