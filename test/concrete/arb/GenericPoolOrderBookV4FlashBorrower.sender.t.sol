// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV4FlashBorrower,
    MinimumOutput,
    OrderBookV4ArbConfigV1
} from "src/concrete/arb/GenericPoolOrderBookV4FlashBorrower.sol";
import {
    OrderV3,
    TakeOrderConfigV3,
    EvaluableV3,
    TakeOrdersConfigV3,
    IInterpreterV3,
    IInterpreterStoreV2
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

contract GenericPoolOrderBookV4FlashBorrowerTest is ArbTest {
    function buildArb(OrderBookV4ArbConfigV1 memory config) internal override returns (address) {
        return address(new GenericPoolOrderBookV4FlashBorrower(config));
    }

    constructor() ArbTest() {}

    function testGenericPoolOrderBookV4FlashBorrowerTakeOrdersSender(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex
    ) public {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV4FlashBorrower(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, ""),
            0,
            abi.encode(iRefundoor, iRefundoor, ""),
            EvaluableV3(iInterpreter, iInterpreterStore, "")
        );
    }

    function testGenericPoolOrderBookV4FlashBorrowerMinimumOutput(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256 minimumOutput,
        uint256 mintAmount
    ) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        minimumOutput = bound(minimumOutput, mintAmount + 1, type(uint256).max);
        iTakerOutput.mint(iArb, mintAmount);

        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        GenericPoolOrderBookV4FlashBorrower(iArb).arb3(
            iOrderBook,
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, ""),
            minimumOutput,
            abi.encode(iRefundoor, iRefundoor, ""),
            EvaluableV3(iInterpreter, iInterpreterStore, "")
        );
    }
}
