// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {StackAllocationMismatch} from "rain.interpreter/error/ErrIntegrity.sol";
import {ExpectedOperand, UnexpectedOperandValue} from "rain.interpreter/error/ErrParse.sol";
import {DEPLOYER_META_PATH} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {OpTest} from "rain.interpreter/../test/abstract/OpTest.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {LibOrderBookSubParserContextFixture} from "test/util/fixture/LibOrderBookSubParserContextFixture.sol";

contract OrderBookSubParserSignedContextTest is OpTest {
    using Strings for address;

    function constructionMetaPath() internal view virtual override returns (string memory) {
        return DEPLOYER_META_PATH;
    }

    /// Test signed-context-0-0
    function testSubParserContextSignedContextHappy0() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        uint256[] memory expectedStack = new uint256[](1);
        expectedStack[0] = uint256(keccak256(bytes("signed-context-0-0")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 0>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-0-0"
        );
    }

    /// Test signed-context-0-1
    function testSubParserContextSignedContextHappy1() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        uint256[] memory expectedStack = new uint256[](1);
        expectedStack[0] = uint256(keccak256(bytes("signed-context-0-1")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 1>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-0-1"
        );
    }

    /// Test signed-context-1-0
    function testSubParserContextSignedContextHappy2() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        uint256[] memory expectedStack = new uint256[](1);
        expectedStack[0] = uint256(keccak256(bytes("signed-context-1-0")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<1 0>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-1-0"
        );
    }

    /// Test signed-context-1-1
    function testSubParserContextSignedContextHappy3() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        uint256[] memory expectedStack = new uint256[](1);
        expectedStack[0] = uint256(keccak256(bytes("signed-context-1-1")));

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<1 1>();")
        );

        checkHappy(
            rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, "signed-context-1-1"
        );
    }

    /// Test signed-context without an operand errors.
    function testSubParserContextSignedContextUnhappyNoOperand() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context();")
        );

        checkUnhappyParse(rainlang, abi.encodeWithSelector(ExpectedOperand.selector));
    }

    /// Test signed-context with too many operands errors.
    function testSubParserContextSignedContextUnhappyTooManyOperands() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        bytes memory rainlang = bytes(
            string.concat(
                "using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 0 0>();"
            )
        );

        checkUnhappyParse(rainlang, abi.encodeWithSelector(UnexpectedOperandValue.selector));
    }

    /// Test signed-context with an input errors.
    function testSubParserContextSignedContextUnhappyInput() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        bytes memory rainlang = bytes(
            string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: signed-context<0 0>(0);")
        );

        checkUnhappyDeploy(rainlang, abi.encodeWithSelector(StackAllocationMismatch.selector, 2, 1));
    }
}
