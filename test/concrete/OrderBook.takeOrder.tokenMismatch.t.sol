// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "test/util/abstract/OrderBookExternalRealTest.sol";
import {TokenMismatch} from "src/concrete/OrderBook.sol";

/// @title OrderBookTakeOrderTokenMismatchTest
/// @notice A test harness for testing the OrderBook takeOrder function.
/// Focuses on the token mismatch case.
contract OrderBookTakeOrderTokenMismatchTest is OrderBookExternalRealTest {
    /// It is only possible to get a token mismatch when there are at least two
    /// orders. This is because `takeOrders` is interactive so we assume that
    /// the caller's desired input and output tokens match the first order they
    /// pass in.
    /// Test a mismatch in the input tokens.
    function testTokenMismatchInputs(
        Order memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        Order memory b,
        uint256 bInputIOIndex,
        uint256 bOutputIOIndex
    ) external {
        vm.assume(a.validInputs.length > 0);
        aInputIOIndex = bound(aInputIOIndex, 0, a.validInputs.length - 1);
        vm.assume(b.validInputs.length > 0);
        bInputIOIndex = bound(bInputIOIndex, 0, b.validInputs.length - 1);
        vm.assume(a.validOutputs.length > 0);
        aOutputIOIndex = bound(aOutputIOIndex, 0, a.validOutputs.length - 1);
        vm.assume(b.validOutputs.length > 0);
        bOutputIOIndex = bound(bOutputIOIndex, 0, b.validOutputs.length - 1);

        // Mismatch on inputs across orders taken.
        vm.assume(a.validInputs[aInputIOIndex].token != b.validInputs[bInputIOIndex].token);
        // Line up outputs so we don't trigger that code path.
        b.validOutputs[bOutputIOIndex].token = a.validOutputs[aOutputIOIndex].token;

        TakeOrderConfig[] memory orders = new TakeOrderConfig[](2);
        orders[0] = TakeOrderConfig(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfig(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, "");
        vm.expectRevert(
            abi.encodeWithSelector(
                TokenMismatch.selector, b.validInputs[bInputIOIndex].token, a.validInputs[aInputIOIndex].token
            )
        );
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// Test a mismatch in the output tokens.
    function testTokenMismatchOutputs(
        Order memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        Order memory b,
        uint256 bInputIOIndex,
        uint256 bOutputIOIndex
    ) external {
        vm.assume(a.validInputs.length > 0);
        aInputIOIndex = bound(aInputIOIndex, 0, a.validInputs.length - 1);
        vm.assume(b.validInputs.length > 0);
        bInputIOIndex = bound(bInputIOIndex, 0, b.validInputs.length - 1);
        vm.assume(a.validOutputs.length > 0);
        aOutputIOIndex = bound(aOutputIOIndex, 0, a.validOutputs.length - 1);
        vm.assume(b.validOutputs.length > 0);
        bOutputIOIndex = bound(bOutputIOIndex, 0, b.validOutputs.length - 1);

        // Mismatch on outputs across orders taken.
        vm.assume(a.validOutputs[aOutputIOIndex].token != b.validOutputs[bOutputIOIndex].token);
        // Line up inputs so we don't trigger that code path.
        b.validInputs[bInputIOIndex].token = a.validInputs[aInputIOIndex].token;

        TakeOrderConfig[] memory orders = new TakeOrderConfig[](2);
        orders[0] = TakeOrderConfig(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfig(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, "");
        vm.expectRevert(
            abi.encodeWithSelector(
                TokenMismatch.selector, b.validOutputs[bOutputIOIndex].token, a.validOutputs[aOutputIOIndex].token
            )
        );
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }
}
