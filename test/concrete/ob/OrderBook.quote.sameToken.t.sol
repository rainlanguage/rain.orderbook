// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Quote, OrderConfigV3, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {TokenSelfTrade} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookQuoteSameTokenTest
contract OrderBookQuoteSameTokenTest is OrderBookExternalRealTest {
    /// Same token for input and output is error.
    /// forge-config: default.fuzz.runs = 10
    function testQuoteSameToken(Quote memory quoteConfig) external {
        vm.assume(quoteConfig.order.validInputs.length > 0);
        vm.assume(quoteConfig.order.validOutputs.length > 0);
        quoteConfig.order.validInputs[0].token = quoteConfig.order.validOutputs[0].token;
        quoteConfig.inputIOIndex = 0;
        quoteConfig.outputIOIndex = 0;
        vm.prank(quoteConfig.order.owner);
        iOrderbook.addOrder2(
            OrderConfigV3({
                evaluable: quoteConfig.order.evaluable,
                validInputs: quoteConfig.order.validInputs,
                validOutputs: quoteConfig.order.validOutputs,
                nonce: quoteConfig.order.nonce,
                secret: bytes32(0),
                meta: ""
            }),
            new TaskV1[](0)
        );
        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        (bool success, uint256 maxOutput, uint256 ioRatio) = iOrderbook.quote(quoteConfig);
        (success, maxOutput, ioRatio);
    }
}
