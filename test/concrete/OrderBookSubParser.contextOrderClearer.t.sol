// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {OrderBookSubParser} from "src/concrete/OrderBookSubParser.sol";
import {IParserV1} from "rain.interpreter/interface/IParserV1.sol";

contract OrderBookSubParserContextOrderClearerTest is OrderBookExternalRealTest {
    using Strings for address;

    function testOrderBookSubParserContextOrderClearerHappy() external {
        OrderBookSubParser orderBookSubParser = new OrderBookSubParser();

        uint256[] memory expectedStack = new uint256[](1);
        expectedStack[0] = uint256(uint160(msg.sender));

        (bytes memory bytecode, uint256[] memory constants) =
        IParserV1(address(iParser)).parse(
            bytes(
                string.concat(
                    "using-words-from ", address(orderBookSubParser).toHexString(), " _: order-clearer();"
                )
            )
        );

    }
}
