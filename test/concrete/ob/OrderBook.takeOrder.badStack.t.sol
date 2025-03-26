// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV4,
    SignedContextV1,
    OrderV4,
    EvaluableV4,
    TaskV2,
    TakeOrdersConfigV4, TakeOrderConfigV4
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {UnsupportedCalculateOutputs} from "src/concrete/ob/OrderBook.sol";

contract OrderBookTakeOrderBadStackTest is OrderBookExternalRealTest {
    function checkBadStack(
        address alice,
        address bob,
        OrderConfigV4 memory config,
        bytes memory rainString,
        uint256 badStackHeight
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        config.evaluable.bytecode = iParserV2.parse2(rainString);

        OrderV4 memory order = OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        TakeOrderConfigV4[] memory takeOrderConfigs = new TakeOrderConfigV4[](1);
        takeOrderConfigs[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV4 memory takeOrdersConfig =
            TakeOrdersConfigV4(0, type(uint256).max, type(uint256).max, takeOrderConfigs, "");

        vm.prank(alice);
        iOrderbook.addOrder3(config, new TaskV2[](0));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, badStackHeight));
        iOrderbook.takeOrders2(takeOrdersConfig);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackEmptyStack(address alice, address bob, OrderConfigV4 memory config) external {
        checkBadStack(alice, bob, config, ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackOneStack(address alice, address bob, OrderConfigV4 memory config) external {
        checkBadStack(alice, bob, config, "_:1;:;", 1);
    }
}
