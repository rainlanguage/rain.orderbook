// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibRouteProcessorOrderBookV3ArbOrderTakerConstants.sol";

import {ArbTest, ArbTestConstructorConfig} from "test/util/abstract/ArbTest.sol";

import "src/concrete/RouteProcessorOrderBookV3ArbOrderTaker.sol";
import "src/interface/unstable/IOrderBookV3.sol";

import "rain.factory/src/interface/ICloneableV2.sol";

contract RouteProcessorOrderBookV3ArbOrderTakerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        (address deployer, DeployerDiscoverableMetaV3ConstructionConfig memory config) =
            buildConstructorConfig(ROUTE_PROCESSOR_ORDER_BOOK_V3_ARB_ORDER_TAKER_META_PATH);
        return ArbTestConstructorConfig(deployer, address(new RouteProcessorOrderBookV3ArbOrderTaker(config)));
    }

    constructor() ArbTest(buildArbTestConstructorConfig()) {
        ICloneableV2(iArb).initialize(
            abi.encode(
                OrderBookV3ArbOrderTakerConfigV1(
                    address(iOrderBook),
                    EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
                    abi.encode(iRefundoor)
                )
            )
        );
    }

    function testTakeOrdersSender(OrderV2 memory order, uint256 inputIOIndex, uint256 outputIOIndex) public {
        TakeOrderConfigV2[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        RouteProcessorOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))), 0
        );
    }

    function testMinimumOutput(
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
        RouteProcessorOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))),
            minimumOutput
        );
    }
}
