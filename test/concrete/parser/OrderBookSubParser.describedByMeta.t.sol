// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";

contract OrderBookSubParserDescribedByMetaV1Test is Test {
    function testOrderBookSubParserDescribedByMetaV1Happy() external {
        bytes memory describedByMeta = vm.readFileBinary("meta/OrderBookSubParserDescribedByMetaV1.rain.meta");
        OrderBookSubParser subParser = new OrderBookSubParser();

        assertEq(keccak256(describedByMeta), subParser.describedByMetaV1());
    }
}
