// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    TaskV2,
    SignedContextV1,
    ClearConfigV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {ClearZeroAmount} from "src/concrete/ob/OrderBook.sol";

contract OrderBookClearZeroAmountTest is OrderBookExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testClearZeroAmount(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configAlice.validOutputs[0].token = address(iToken0);
        configAlice.validInputs[0].token = address(iToken1);
        configBob.validInputs[0].token = configAlice.validOutputs[0].token;
        configBob.validOutputs[0].token = configAlice.validInputs[0].token;

        bytes memory rainstring = bytes(string.concat("_ _:0 0;", ":;"));
        bytes memory bytecode = iParserV2.parse2(rainstring);
        configAlice.evaluable.bytecode = bytecode;
        configBob.evaluable.bytecode = bytecode;

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV4 memory orderBob =
            OrderV4(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder3(configAlice, new TaskV2[](0));
        vm.prank(bob);
        iOrderbook.addOrder3(configBob, new TaskV2[](0));

        vm.expectRevert(ClearZeroAmount.selector);
        iOrderbook.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
