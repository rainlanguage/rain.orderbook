// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3,
    OrderV3,
    EvaluableV3,
    ClearConfig,
    SignedContextV1,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

contract OrderBookClearSameTokenTest is OrderBookExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testClearSameToken(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {}
}
