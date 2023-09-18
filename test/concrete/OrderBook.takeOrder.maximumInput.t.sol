// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Order, SignedContextV1, TakeOrderConfig, TakeOrdersConfigV2, ZeroMaximumInput} from "src/interface/unstable/IOrderBookV3.sol";

contract OrderBookTakeOrderMaximumInputTest is OrderBookExternalRealTest {

    /// If there is some live order(s) but the maxTakerInput is zero we error as
    /// the caller has full control over this, and it would cause none of the
    /// orders to be taken.
    function testTakeOrderNoopZeroMaxTakerInput(Order memory order, SignedContextV1 memory signedContext) external {
        vm.assume(order.validInputs.length > 0);
        vm.assume(order.validOutputs.length > 0);
        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        orders[0] = TakeOrderConfig(order, 0, 0, signedContexts);
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, 0, type(uint256).max, orders, "");
        vm.expectRevert(ZeroMaximumInput.selector);
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }

}