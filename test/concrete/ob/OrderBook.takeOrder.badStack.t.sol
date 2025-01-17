// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV3,
    SignedContextV1,
    OrderV3,
    EvaluableV3,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {TakeOrdersConfigV3, TakeOrderConfigV3} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {UnsupportedCalculateOutputs} from "src/concrete/ob/OrderBook.sol";

contract OrderBookTakeOrderBadStackTest is OrderBookExternalRealTest {
    function checkBadStack(
        address alice,
        address bob,
        OrderConfigV3 memory config,
        bytes memory rainString,
        uint256 badStackHeight
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        config.evaluable.bytecode = iParserV2.parse2(rainString);

        OrderV3 memory order = OrderV3(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        TakeOrderConfigV3[] memory takeOrderConfigs = new TakeOrderConfigV3[](1);
        takeOrderConfigs[0] = TakeOrderConfigV3(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV3 memory takeOrdersConfig =
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, takeOrderConfigs, "");

        vm.prank(alice);
        iOrderbook.addOrder2(config, new TaskV1[](0));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, badStackHeight));
        iOrderbook.takeOrders2(takeOrdersConfig);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackEmptyStack(address alice, address bob, OrderConfigV3 memory config) external {
        checkBadStack(alice, bob, config, ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackOneStack(address alice, address bob, OrderConfigV3 memory config) external {
        checkBadStack(alice, bob, config, "_:1;:;", 1);
    }
}
