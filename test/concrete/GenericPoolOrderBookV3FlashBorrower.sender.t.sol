// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "test/util/abstract/ArbTest.sol";

import {GENERIC_POOL_ORDER_BOOK_V3_FLASH_BORROWER_META_PATH} from
    "test/util/lib/LibGenericPoolOrderBookV3FlashBorrowerConstants.sol";

import {
    GenericPoolOrderBookV3FlashBorrower,
    DeployerDiscoverableMetaV3ConstructionConfig,
    CALLER_META_HASH as GENERIC_POOL_ORDER_BOOK_V3_FLASH_BORROWER_CALLER_META_HASH,
    MinimumOutput,
    ICloneableV2,
    OrderBookV3FlashBorrowerConfigV2
} from "src/concrete/GenericPoolOrderBookV3FlashBorrower.sol";
import {
    OrderV2,
    TakeOrderConfigV2,
    EvaluableConfigV3,
    TakeOrdersConfigV2,
    IExpressionDeployerV3
} from "src/interface/unstable/IOrderBookV3.sol";

contract GenericPoolOrderBookV3FlashBorrowerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        (address deployer, DeployerDiscoverableMetaV3ConstructionConfig memory config) = buildConstructorConfig(
            GENERIC_POOL_ORDER_BOOK_V3_FLASH_BORROWER_META_PATH,
            GENERIC_POOL_ORDER_BOOK_V3_FLASH_BORROWER_CALLER_META_HASH
        );
        return ArbTestConstructorConfig(deployer, address(new GenericPoolOrderBookV3FlashBorrower(config)));
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
