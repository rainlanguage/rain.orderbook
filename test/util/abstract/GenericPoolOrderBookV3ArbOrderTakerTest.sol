// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV3ArbOrderTaker,
    DeployerDiscoverableMetaV3ConstructionConfig,
    OrderBookV3ArbOrderTakerConfigV1,
    CALLER_META_HASH as GENERIC_POOL_ORDER_BOOK_V3_ARB_ORDER_TAKER_CALLER_META_HASH
} from "src/concrete/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {
    OrderV2,
    EvaluableConfigV3,
    IExpressionDeployerV3,
    TakeOrderConfigV2,
    TakeOrdersConfigV2
} from "src/interface/unstable/IOrderBookV3.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";
import {GENERIC_POOL_ORDER_BOOK_V3_ARB_ORDER_TAKER_META_PATH} from
    "test/util/lib/LibGenericPoolOrderBookV3ArbOrderTakerConstants.sol";

contract GenericPoolOrderBookV3ArbOrderTakerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        (address deployer, DeployerDiscoverableMetaV3ConstructionConfig memory config) = buildConstructorConfig(
            GENERIC_POOL_ORDER_BOOK_V3_ARB_ORDER_TAKER_META_PATH,
            GENERIC_POOL_ORDER_BOOK_V3_ARB_ORDER_TAKER_CALLER_META_HASH
        );
        return ArbTestConstructorConfig(deployer, address(new GenericPoolOrderBookV3ArbOrderTaker(config)));
    }

    constructor() ArbTest(buildArbTestConstructorConfig()) {
        ICloneableV2(iArb).initialize(
            abi.encode(
                OrderBookV3ArbOrderTakerConfigV1(
                    address(iOrderBook), EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)), ""
                )
            )
        );
    }
}
