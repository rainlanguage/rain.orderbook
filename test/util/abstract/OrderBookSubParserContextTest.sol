// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {StackAllocationMismatch} from "rain.interpreter/error/ErrIntegrity.sol";
import {OpTest, StackItem} from "rain.interpreter/../test/abstract/OpTest.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {LibOrderBookSubParserContextFixture} from "test/util/fixture/LibOrderBookSubParserContextFixture.sol";

abstract contract OrderBookSubParserContextTest is OpTest {
    using Strings for address;

    function word() internal pure virtual returns (string memory);

    function testSubParserContextHappy() external {
        string memory w = word();
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes(w)));

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: ", w, "();"));

        checkHappy(rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, w);
    }

    function testSubParserContextUnhappyDisallowedOperand() external {
        string memory w = word();
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: ", w, "<1>();"));

        checkDisallowedOperand(rainlang);
    }

    function testSubParserContextUnhappyDisallowedInputs() external {
        string memory w = word();
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: ", w, "(1);"));

        checkUnhappyParse2(rainlang, abi.encodeWithSelector(StackAllocationMismatch.selector, 2, 1));
    }
}
