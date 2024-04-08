// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ArbTest, ArbTestConstructorConfig} from "./ArbTest.sol";
import {
    GenericPoolOrderBookV4ArbOrderTaker,
    OrderBookV4ArbOrderTakerConfigV1
} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";
import {
    OrderV3,
    EvaluableV3,
    IExpressionDeployerV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";

contract GenericPoolOrderBookV4ArbOrderTakerTest is ArbTest {
    function buildArbTestConstructorConfig() internal returns (ArbTestConstructorConfig memory) {
        address deployer = buildConstructorConfig();
        return ArbTestConstructorConfig(deployer, address(new GenericPoolOrderBookV4ArbOrderTaker()));
    }

    constructor() ArbTest(buildArbTestConstructorConfig()) {
        ICloneableV2(iArb).initialize(
            abi.encode(
                OrderBookV4ArbOrderTakerConfigV2(
                    address(iOrderBook), EvaluableV3(IInterpreterV3(address(0)), IInterpreterStoreV2(address(0)), ""), ""
                )
            )
        );
    }
}
