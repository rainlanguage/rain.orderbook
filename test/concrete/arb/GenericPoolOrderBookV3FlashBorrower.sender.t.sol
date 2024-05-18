// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV3FlashBorrower,
    MinimumOutput,
    OrderBookV3ArbConfigV1
} from "src/concrete/arb/GenericPoolOrderBookV3FlashBorrower.sol";
import {
    OrderV2,
    TakeOrderConfigV2,
    EvaluableConfigV3,
    TakeOrdersConfigV2,
    IExpressionDeployerV3
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";

contract GenericPoolOrderBookV3FlashBorrowerTest is ArbTest {
    function buildArb(OrderBookV3ArbConfigV1 memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV3FlashBorrower(config));
    }

    constructor() ArbTest() {}

    function testGenericPoolOrderBookV3FlashBorrowerTakeOrdersSender(
        OrderV2 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex
    ) public {
        TakeOrderConfigV2[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV3FlashBorrower(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, ""),
            0,
            abi.encode(iRefundoor, iRefundoor, "")
        );
    }

    function testGenericPoolOrderBookV3FlashBorrowerMinimumOutput(
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
        GenericPoolOrderBookV3FlashBorrower(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, ""),
            minimumOutput,
            abi.encode(iRefundoor, iRefundoor, "")
        );
    }
}
