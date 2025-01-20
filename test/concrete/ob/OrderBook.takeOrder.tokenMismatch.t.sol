// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderV3,
    IO,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    EvaluableV3,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {TokenMismatch} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookTakeOrderTokenMismatchTest
/// @notice A test harness for testing the OrderBook takeOrder function.
/// Focuses on the token mismatch case.
contract OrderBookTakeOrderTokenMismatchTest is OrderBookExternalRealTest {
    /// It is only possible to get a token mismatch when there are at least two
    /// orders. This is because `takeOrders` is interactive so we assume that
    /// the caller's desired input and output tokens match the first order they
    /// pass in.
    /// Test a mismatch in the input tokens.
    /// forge-config: default.fuzz.runs = 10
    function testTokenMismatchInputs(
        OrderV3 memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        OrderV3 memory b,
        uint256 bInputIOIndex,
        uint256 bOutputIOIndex,
        uint256 maxTakerInput,
        uint256 maxIORatio
    ) external {
        vm.assume(a.validInputs.length > 0);
        aInputIOIndex = bound(aInputIOIndex, 0, a.validInputs.length - 1);
        vm.assume(b.validInputs.length > 0);
        bInputIOIndex = bound(bInputIOIndex, 0, b.validInputs.length - 1);
        vm.assume(a.validOutputs.length > 0);
        aOutputIOIndex = bound(aOutputIOIndex, 0, a.validOutputs.length - 1);
        vm.assume(b.validOutputs.length > 0);
        bOutputIOIndex = bound(bOutputIOIndex, 0, b.validOutputs.length - 1);
        maxTakerInput = bound(maxTakerInput, 1, type(uint256).max);

        vm.assume(a.validInputs[aInputIOIndex].token != a.validOutputs[aOutputIOIndex].token);

        // Mismatch on inputs across orders taken.
        vm.assume(a.validInputs[aInputIOIndex].token != b.validInputs[bInputIOIndex].token);
        // Line up outputs so we don't trigger that code path.
        b.validOutputs[bOutputIOIndex].token = a.validOutputs[aOutputIOIndex].token;

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](2);
        orders[0] = TakeOrderConfigV3(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV3(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV3 memory config = TakeOrdersConfigV3(0, maxTakerInput, maxIORatio, orders, "");
        vm.expectRevert(abi.encodeWithSelector(TokenMismatch.selector));
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders2(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// Test a mismatch in the output tokens.
    /// forge-config: default.fuzz.runs = 10
    function testTokenDecimalMismatchOutputs(
        OrderV3 memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        OrderV3 memory b,
        uint256 bInputIOIndex,
        uint256 bOutputIOIndex,
        uint256 maxTakerInput,
        uint256 maxIORatio
    ) external {
        vm.assume(a.validInputs.length > 0);
        aInputIOIndex = bound(aInputIOIndex, 0, a.validInputs.length - 1);
        vm.assume(b.validInputs.length > 0);
        bInputIOIndex = bound(bInputIOIndex, 0, b.validInputs.length - 1);
        vm.assume(a.validOutputs.length > 0);
        aOutputIOIndex = bound(aOutputIOIndex, 0, a.validOutputs.length - 1);
        vm.assume(b.validOutputs.length > 0);
        bOutputIOIndex = bound(bOutputIOIndex, 0, b.validOutputs.length - 1);
        maxTakerInput = bound(maxTakerInput, 1, type(uint256).max);

        vm.assume(a.validOutputs[aOutputIOIndex].token != a.validInputs[aInputIOIndex].token);

        // Mismatch on outputs across orders taken.
        vm.assume(a.validOutputs[aOutputIOIndex].token != b.validOutputs[bOutputIOIndex].token);
        // Line up inputs so we don't trigger that code path.
        b.validInputs[bInputIOIndex].token = a.validInputs[aInputIOIndex].token;

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](2);
        orders[0] = TakeOrderConfigV3(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV3(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV3 memory config = TakeOrdersConfigV3(0, maxTakerInput, maxIORatio, orders, "");
        vm.expectRevert(abi.encodeWithSelector(TokenMismatch.selector));
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders2(config);
        (totalTakerInput, totalTakerOutput);
    }
}
