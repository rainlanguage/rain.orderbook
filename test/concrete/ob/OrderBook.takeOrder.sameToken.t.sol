// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3,
    OrderV3,
    TaskV1,
    TakeOrderConfigV3,
    SignedContextV1,
    TakeOrdersConfigV3
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {TokenSelfTrade} from "src/concrete/ob/OrderBook.sol";

contract OrderBookTakeOrderSameTokenTest is OrderBookExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testTakeOrderSameToken(address alice, OrderConfigV3 memory configAlice) external {
        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        configAlice.validInputs[0].token = address(0);
        configAlice.validOutputs[0].token = address(0);

        OrderV3 memory orderAlice =
            OrderV3(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);

        vm.prank(alice);
        iOrderbook.addOrder2(configAlice, new TaskV1[](0));

        TakeOrderConfigV3[] memory takeOrders = new TakeOrderConfigV3[](1);
        takeOrders[0] = TakeOrderConfigV3({
            order: orderAlice,
            inputIOIndex: 0,
            outputIOIndex: 0,
            signedContext: new SignedContextV1[](0)
        });

        TakeOrdersConfigV3 memory takeOrdersConfig = TakeOrdersConfigV3({
            minimumInput: 0,
            maximumInput: type(uint256).max,
            maximumIORatio: type(uint256).max,
            orders: takeOrders,
            data: ""
        });

        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        iOrderbook.takeOrders2(takeOrdersConfig);
    }
}
