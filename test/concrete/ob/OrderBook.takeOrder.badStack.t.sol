// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV4,
    SignedContextV1,
    OrderV4,
    EvaluableV4,
    TaskV2,
    TakeOrdersConfigV4,
    TakeOrderConfigV4
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {UnsupportedCalculateOutputs} from "src/concrete/ob/OrderBook.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract OrderBookTakeOrderBadStackTest is OrderBookExternalRealTest {
    function checkBadStack(
        address alice,
        address bob,
        OrderConfigV4 memory config,
        bytes memory rainString,
        uint256 badStackHeight
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        config.evaluable.bytecode = iParserV2.parse2(rainString);

        OrderV4 memory order = OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        TakeOrderConfigV4[] memory takeOrderConfigs = new TakeOrderConfigV4[](1);
        takeOrderConfigs[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV4 memory takeOrdersConfig = TakeOrdersConfigV4(
            LibDecimalFloat.packLossless(0, 0),
            LibDecimalFloat.packLossless(type(int224).max, 0),
            LibDecimalFloat.packLossless(type(int224).max, 0),
            takeOrderConfigs,
            ""
        );
        config.validInputs[0].token = address(iToken0);
        config.validOutputs[0].token = address(iToken1);

        vm.prank(alice);
        iOrderbook.addOrder3(config, new TaskV2[](0));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, badStackHeight));
        iOrderbook.takeOrders3(takeOrdersConfig);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackEmptyStack(address alice, address bob, OrderConfigV4 memory config) external {
        checkBadStack(alice, bob, config, ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackOneStack(address alice, address bob, OrderConfigV4 memory config) external {
        checkBadStack(alice, bob, config, "_:1;:;", 1);
    }
}
