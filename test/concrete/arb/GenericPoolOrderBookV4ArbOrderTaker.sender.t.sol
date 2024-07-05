// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {GenericPoolOrderBookV4ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV4ArbOrderTakerTest.sol";

import {
    GenericPoolOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV1,
    MinimumOutput
} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";
import {
    OrderV3,
    EvaluableV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

contract GenericPoolOrderBookV4ArbOrderTakerSenderTest is GenericPoolOrderBookV4ArbOrderTakerTest {
    function testGenericPoolTakeOrdersSender(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV4ArbOrderTaker(iArb).arb2(
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            0,
            EvaluableV3(iInterpreter, iInterpreterStore, "")
        );
    }

    function testGenericPoolMinimumOutput(
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
        GenericPoolOrderBookV4ArbOrderTaker(iArb).arb2(
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            minimumOutput,
            EvaluableV3(iInterpreter, iInterpreterStore, "")
        );
    }
}
