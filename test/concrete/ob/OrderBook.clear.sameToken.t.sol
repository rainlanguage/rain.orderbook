// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
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
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {TokenSelfTrade} from "src/concrete/ob/OrderBook.sol";

contract OrderBookClearSameTokenTest is OrderBookExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testClearSameToken(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configAlice.validInputs[0].token = address(0);
        configAlice.validOutputs[0].token = address(0);
        configBob.validInputs[0].token = address(0);
        configBob.validOutputs[0].token = address(0);

        OrderV3 memory orderAlice =
            OrderV3(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV3 memory orderBob =
            OrderV3(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder2(configAlice, new TaskV1[](0));

        vm.prank(bob);
        iOrderbook.addOrder2(configBob, new TaskV1[](0));

        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        iOrderbook.clear2(
            orderAlice, orderBob, ClearConfig(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
