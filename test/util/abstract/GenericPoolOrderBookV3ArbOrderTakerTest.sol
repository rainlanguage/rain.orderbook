// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV3ArbOrderTaker,
    OrderBookV3ArbOrderTakerConfigV1
} from "src/concrete/arb/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {
    OrderV2,
    EvaluableConfigV3,
    IExpressionDeployerV3,
    TakeOrderConfigV2,
    TakeOrdersConfigV2
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";

contract GenericPoolOrderBookV3ArbOrderTakerTest is ArbTest {
    // function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
    //     address deployer = buildConstructorConfig();
    //     return ArbTestConstructorConfig(deployer, address(new GenericPoolOrderBookV3ArbOrderTaker()));
    // }

    function buildArb(OrderBookV3ArbOrderTakerConfigV1 memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV3ArbOrderTaker(config));
    }

    constructor() ArbTest() {
    }
}
