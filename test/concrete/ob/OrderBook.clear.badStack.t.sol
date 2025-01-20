// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV3,
    OrderV3,
    EvaluableV3,
    ClearConfig,
    SignedContextV1,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {UnsupportedCalculateOutputs} from "src/concrete/ob/OrderBook.sol";

contract OrderBookClearOrderBadStackTest is OrderBookExternalRealTest {
    function checkBadStack(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob,
        bytes memory rainStringAlice,
        bytes memory rainStringBob,
        uint256 badStackHeight
    ) internal {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configBob.validOutputs[0] = configAlice.validInputs[0];
        configBob.validInputs[0] = configAlice.validOutputs[0];

        configAlice.evaluable.bytecode = iParserV2.parse2(rainStringAlice);
        configBob.evaluable.bytecode = iParserV2.parse2(rainStringBob);

        OrderV3 memory orderAlice =
            OrderV3(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV3 memory orderBob =
            OrderV3(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder2(configAlice, new TaskV1[](0));

        vm.prank(bob);
        iOrderbook.addOrder2(configBob, new TaskV1[](0));

        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, badStackHeight));
        iOrderbook.clear2(
            orderAlice, orderBob, ClearConfig(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackEmptyStack(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, ":;:;", ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneStack(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_:1;:;", "_:1;:;", 1);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneEmpty(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_ _:1 1;:;", ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOtherEmpty(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, ":;:;", "_ _:1 1;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneOne(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_ _:1 1;:;", "_:1;:;", 1);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneOtherOne(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_:1;:;", "_ _:1 1;:;", 1);
    }
}
