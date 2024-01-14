// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {GenericPoolOrderBookV3ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV3ArbOrderTakerTest.sol";

import {
    GenericPoolOrderBookV3ArbOrderTaker,
    OrderBookV3ArbOrderTakerConfigV1,
    MinimumOutput
} from "src/concrete/arb/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";
import {
    OrderV2,
    EvaluableConfigV3,
    IExpressionDeployerV3,
    TakeOrderConfigV2,
    TakeOrdersConfigV2
} from "src/interface/unstable/IOrderBookV3.sol";

contract GenericPoolOrderBookV3ArbOrderTakerSenderTest is GenericPoolOrderBookV3ArbOrderTakerTest {
    function testGenericPoolTakeOrdersSender(OrderV2 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV2[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            0
        );
    }

    function testGenericPoolMinimumOutput(
        OrderV2 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256 minimumOutput,
        uint256 mintAmount
    ) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        minimumOutput = bound(minimumOutput, mintAmount + 1, type(uint256).max);
        iTakerOutput.mint(iArb, mintAmount);

        TakeOrderConfigV2[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        GenericPoolOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            minimumOutput
        );
    }
}
