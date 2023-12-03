// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "./ArbTest.sol";
import {
    RouteProcessorOrderBookV3ArbOrderTaker,
    DeployerDiscoverableMetaV3ConstructionConfig,
    OrderBookV3ArbOrderTakerConfigV1
} from "src/concrete/RouteProcessorOrderBookV3ArbOrderTaker.sol";
import {
    OrderV2,
    EvaluableConfigV3,
    IExpressionDeployerV3,
    TakeOrderConfigV2,
    TakeOrdersConfigV2
} from "src/interface/unstable/IOrderBookV3.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";
import {ROUTE_PROCESSOR_ORDER_BOOK_V3_ARB_ORDER_TAKER_META_PATH} from
    "test/util/lib/LibRouteProcessorOrderBookV3ArbOrderTakerConstants.sol";

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
}
