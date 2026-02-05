// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {StackAllocationMismatch} from "rain.interpreter/error/ErrIntegrity.sol";
import {ExpectedOperand, UnexpectedOperandValue} from "rain.interpreter/error/ErrParse.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {OpTest, StackItem} from "rain.interpreter/../test/abstract/OpTest.sol";
import {OrderBookV6SubParser} from "src/concrete/parser/OrderBookV6SubParser.sol";
import {LibOrderBookSubParserContextFixture} from "test/util/fixture/LibOrderBookSubParserContextFixture.sol";

contract OrderBookV6SubParserSignedContextTest is OpTest {
    using Strings for address;

    /// Test signed-context-0-0
    function testSubParserContextSignedContextHappy0() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes("signed-context-0-0")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 0>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-0-0"
        );
    }

    /// Test signed-context-0-1
    function testSubParserContextSignedContextHappy1() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes("signed-context-0-1")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 1>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-0-1"
        );
    }

    /// Test signed-context-1-0
    function testSubParserContextSignedContextHappy2() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes("signed-context-1-0")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<1 0>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-1-0"
        );
    }

    /// Test signed-context-1-1
    function testSubParserContextSignedContextHappy3() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes("signed-context-1-1")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<1 1>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-1-1"
        );
    }

    /// Test signed-context without an operand errors.
    function testSubParserContextSignedContextUnhappyNoOperand() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context();")
        );

        checkUnhappyParse(rainlang, abi.encodeWithSelector(ExpectedOperand.selector));
    }

    /// Test signed-context with too many operands errors.
    function testSubParserContextSignedContextUnhappyTooManyOperands() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        bytes memory rainlang = bytes(
            string.concat(
                "using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 0 0>();"
            )
        );

        checkUnhappyParse(rainlang, abi.encodeWithSelector(UnexpectedOperandValue.selector));
    }

    /// Test signed-context with an input errors.
    function testSubParserContextSignedContextUnhappyInput() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 0>(0);")
        );

        checkUnhappyParse2(rainlang, abi.encodeWithSelector(StackAllocationMismatch.selector, 2, 1));
    }
}
