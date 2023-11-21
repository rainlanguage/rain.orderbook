// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "test/util/abstract/ArbTest.sol";
import "openzeppelin-contracts/contracts/proxy/Clones.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibGenericPoolOrderBookV3ArbOrderTakerConstants.sol";

import "src/concrete/GenericPoolOrderBookV3ArbOrderTaker.sol";
import "src/interface/unstable/IOrderBookV3.sol";

contract GenericPoolOrderBookV3ArbOrderTakerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        (address deployer, DeployerDiscoverableMetaV3ConstructionConfig memory config) =
            buildConstructorConfig(GENERIC_POOL_ORDER_BOOK_V3_ARB_ORDER_TAKER_META_PATH);
        return ArbTestConstructorConfig(deployer, address(new GenericPoolOrderBookV3ArbOrderTaker(config)));
    }

    constructor() ArbTest(buildArbTestConstructorConfig()) {
        ICloneableV2(iArb).initialize(
            abi.encode(
                OrderBookV3ArbOrderTakerConfigV1(
                    address(iOrderBook), EvaluableConfigV2(IExpressionDeployerV3(address(0)), "", new uint256[](0)), ""
                )
            )
        );
    }

    function testTakeOrdersSender(Order memory order, uint256 inputIOIndex, uint256 outputIOIndex) public {
        TakeOrderConfig[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            0
        );
    }

    function testMinimumOutput(
        Order memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256 minimumOutput,
        uint256 mintAmount
    ) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        minimumOutput = bound(minimumOutput, mintAmount + 1, type(uint256).max);
        iTakerOutput.mint(iArb, mintAmount);

        TakeOrderConfig[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        GenericPoolOrderBookV3ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(iRefundoor, iRefundoor, "")),
            minimumOutput
        );
    }
}
