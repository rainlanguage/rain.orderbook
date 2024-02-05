// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV3FlashBorrower,
    MinimumOutput,
    ICloneableV2,
    OrderBookV3FlashBorrowerConfigV2
} from "src/concrete/arb/GenericPoolOrderBookV3FlashBorrower.sol";
import {
    OrderV2,
    TakeOrderConfigV2,
    EvaluableConfigV3,
    TakeOrdersConfigV2,
    IExpressionDeployerV3
} from "src/interface//IOrderBookV3.sol";

contract GenericPoolOrderBookV3FlashBorrowerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        address deployer = buildConstructorConfig();
        address iArb = address(new GenericPoolOrderBookV3FlashBorrower());
        vm.label(iArb, "iArb");
        return ArbTestConstructorConfig(deployer, iArb);
    }

    constructor() ArbTest(buildArbTestConstructorConfig()) {
        ICloneableV2(iArb).initialize(
            abi.encode(
                OrderBookV3FlashBorrowerConfigV2(
                    address(iOrderBook), EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)), ""
                )
            )
        );
    }

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
