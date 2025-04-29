// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    TaskV2,
    TakeOrderConfigV4,
    SignedContextV1,
    TakeOrdersConfigV4
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {TokenSelfTrade} from "src/concrete/ob/OrderBook.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

contract OrderBookTakeOrderSameTokenTest is OrderBookExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testTakeOrderSameToken(address alice, OrderConfigV4 memory configAlice) external {
        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        configAlice.validInputs[0].token = address(0);
        configAlice.validOutputs[0].token = address(0);

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);

        vm.prank(alice);
        iOrderbook.addOrder3(configAlice, new TaskV2[](0));

        TakeOrderConfigV4[] memory takeOrders = new TakeOrderConfigV4[](1);
        takeOrders[0] = TakeOrderConfigV4({
            order: orderAlice,
            inputIOIndex: 0,
            outputIOIndex: 0,
            signedContext: new SignedContextV1[](0)
        });

        TakeOrdersConfigV4 memory takeOrdersConfig = TakeOrdersConfigV4({
            minimumInput: LibDecimalFloat.packLossless(0, 0),
            maximumInput: LibDecimalFloat.packLossless(type(int256).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int256).max, 0),
            orders: takeOrders,
            data: ""
        });

        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        iOrderbook.takeOrders3(takeOrdersConfig);
    }
}
