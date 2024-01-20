// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "./ArbTest.sol";
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
} from "src/interface/unstable/IOrderBookV3.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";

contract GenericPoolOrderBookV3ArbOrderTakerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        address deployer = buildConstructorConfig();
        return ArbTestConstructorConfig(deployer, address(new GenericPoolOrderBookV3ArbOrderTaker()));
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
