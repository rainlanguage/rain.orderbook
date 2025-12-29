// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest, Vm} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    SignedContextV1,
    IOrderBookV6
} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract OrderBookV6TakeOrderMaximumOutputTest is OrderBookV6ExternalRealTest {
    /// It should be possible to take an order with zero maximum output.
    function testTakeOrderMaximumOutputZero(OrderV4 memory order, SignedContextV1 memory signedContext) external {
        vm.assume(order.validInputs.length > 0);
        vm.assume(order.validOutputs.length > 0);
        order.validInputs[0].token = address(iToken0);
        order.validOutputs[0].token = address(iToken1);
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        orders[0] = TakeOrderConfigV4({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: signedContexts});
        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            orders: orders,
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIORatio: LibDecimalFloat.packLossless(1, 0),
            IOIsInput: false,
            data: ""
        });
        vm.expectRevert(IOrderBookV6.ZeroMaximumIO.selector);
        (Float totalTakerInput, Float totalTakerOutput) = iOrderbook.takeOrders4(config);
        (totalTakerInput, totalTakerOutput);
    }
}
