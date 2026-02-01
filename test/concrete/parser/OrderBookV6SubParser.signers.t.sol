// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {StackAllocationMismatch} from "rain.interpreter/error/ErrIntegrity.sol";
import {ExpectedOperand, UnexpectedOperandValue} from "rain.interpreter/error/ErrParse.sol";
import {OpTest, StackItem} from "rain.interpreter/../test/abstract/OpTest.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {OrderBookV6SubParser} from "src/concrete/parser/OrderBookV6SubParser.sol";
import {LibOrderBookSubParserContextFixture} from "test/util/fixture/LibOrderBookSubParserContextFixture.sol";

contract OrderBookV6SubParserSignersTest is OpTest {
    using Strings for address;

    /// Test signer-0
    function testSubParserContextSignerHappy0() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes("signer-0")));

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signer<0>();"));

        checkHappy(rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signer-0");
    }

    /// Test signer-1
    function testSubParserContextSignerHappy1() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes("signer-1")));

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signer<1>();"));

        checkHappy(rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signer-1");
    }

    /// Test signer without an operand errors.
    function testSubParserContextSignerUnhappyNoOperand() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signer();"));

        checkUnhappyParse(rainlang, abi.encodeWithSelector(ExpectedOperand.selector));
    }

    /// Test signer with too many operands errors.
    function testSubParserContextSignerUnhappyTooManyOperands() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signer<0 1>();"));

        checkUnhappyParse(rainlang, abi.encodeWithSelector(UnexpectedOperandValue.selector));
    }

    /// Test signer with an input errors.
    function testSubParserContextSignerUnhappyInput() external {
        OrderBookV6SubParser orderBookSubParser = new OrderBookV6SubParser();

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signer<0>(0);"));

        checkUnhappyParse2(rainlang, abi.encodeWithSelector(StackAllocationMismatch.selector, 2, 1));
    }
}
