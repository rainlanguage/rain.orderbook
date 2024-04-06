// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest} from "./ArbTest.sol";
import {
    RouteProcessorOrderBookV3ArbOrderTaker,
    OrderBookV3ArbOrderTakerConfigV1
} from "src/concrete/arb/RouteProcessorOrderBookV3ArbOrderTaker.sol";
import {
    OrderV2,
    EvaluableConfigV3,
    IExpressionDeployerV3,
    TakeOrderConfigV2,
    TakeOrdersConfigV2
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";

contract RouteProcessorOrderBookV3ArbOrderTakerTest is ArbTest {
    // function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
    //     address deployer = buildConstructorConfig();
    //     return ArbTestConstructorConfig(
    //         deployer,
    //         address(
    //             new RouteProcessorOrderBookV3ArbOrderTaker(
    //                 OrderBookV3ArbOrderTakerConfigV1(
    //                     address(iOrderBook),
    //                     EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
    //                     abi.encode(iRefundoor)
    //                 )
    //             )
    //         )
    //     );
    // }

    function buildArb(OrderBookV3ArbOrderTakerConfigV1 memory config) internal override returns (address) {
        return address(
            new RouteProcessorOrderBookV3ArbOrderTaker(config)
        );
    }

    constructor() ArbTest() {}
}
