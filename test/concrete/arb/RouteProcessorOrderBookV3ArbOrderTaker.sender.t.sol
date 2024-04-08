// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {RouteProcessorOrderBookV3ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV3ArbOrderTakerTest.sol";
import {
    OrderV3,
    EvaluableConfigV3,
    IExpressionDeployerV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";
import {
    RouteProcessorOrderBookV3ArbOrderTaker,
    OrderBookV3ArbOrderTakerConfigV1,
    MinimumOutput
} from "src/concrete/arb/RouteProcessorOrderBookV3ArbOrderTaker.sol";

contract RouteProcessorOrderBookV3ArbOrderTakerSenderTest is RouteProcessorOrderBookV3ArbOrderTakerTest {
    function testRouteProcessorTakeOrdersSender(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        RouteProcessorOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))), 0
        );
    }

    function testRouteProcessorMinimumOutput(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256 minimumOutput,
        uint256 mintAmount
    ) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        minimumOutput = bound(minimumOutput, mintAmount + 1, type(uint256).max);
        iTakerOutput.mint(iArb, mintAmount);

        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        RouteProcessorOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))),
            minimumOutput
        );
    }
}
