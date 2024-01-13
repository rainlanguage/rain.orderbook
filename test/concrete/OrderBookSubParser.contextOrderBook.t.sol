// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {DEPLOYER_META_PATH} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OpTest} from "rain.interpreter/../test/util/abstract/OpTest.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {OrderBookSubParser} from "src/concrete/OrderBookSubParser.sol";
import {IParserV1} from "rain.interpreter/interface/IParserV1.sol";

contract OrderBookSubParserContextOrderBookTest is OpTest {
    using Strings for address;

    function constructionMetaPath() internal view virtual override returns (string memory) {
        return DEPLOYER_META_PATH;
    }

    function testOrderBookSubParserContextOrderBookHappy() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        uint256[] memory expectedStack = new uint256[](1);
        expectedStack[0] = uint256(uint160(address(this)));

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", address(orderBookSubParser).toHexString(), " _: orderbook();"));

        checkHappy(rainlang, expectedStack, "orderbook");
    }
}
