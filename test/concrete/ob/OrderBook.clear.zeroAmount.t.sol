// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3,
    OrderV3,
    TaskV1,
    SignedContextV1,
    ClearConfig
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {ClearZeroAmount} from "src/concrete/ob/OrderBook.sol";

contract OrderBookClearZeroAmountTest is OrderBookExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testClearZeroAmount(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configBob.validInputs[0].token = configAlice.validOutputs[0].token;
        configBob.validOutputs[0].token = configAlice.validInputs[0].token;
        configBob.validInputs[0].decimals = configAlice.validOutputs[0].decimals;
        configBob.validOutputs[0].decimals = configAlice.validInputs[0].decimals;

        bytes memory rainstring = bytes(string.concat("_ _:0 0;", ":;"));
        bytes memory bytecode = iParserV2.parse2(rainstring);
        configAlice.evaluable.bytecode = bytecode;
        configBob.evaluable.bytecode = bytecode;

        OrderV3 memory orderAlice =
            OrderV3(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV3 memory orderBob =
            OrderV3(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder2(configAlice, new TaskV1[](0));
        vm.prank(bob);
        iOrderbook.addOrder2(configBob, new TaskV1[](0));

        vm.expectRevert(ClearZeroAmount.selector);
        iOrderbook.clear2(
            orderAlice, orderBob, ClearConfig(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
