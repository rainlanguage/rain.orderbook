// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {TokenDecimalsMismatch} from "src/concrete/ob/OrderBook.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";
import {TakeOrderConfigV3, TakeOrdersConfigV3, OrderV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

/// @title OrderBookTakeOrderTokenMismatchDecimalsTest
/// @notice A test harness for testing the OrderBook takeOrder function.
/// Focuses on the token decimals mismatch case.
contract OrderBookTakeOrderTokenMismatchDecimalsTest is OrderBookExternalRealTest {
    /// It is only possible to get a token mismatch when there are at least two
    /// orders. This is because `takeOrders` is interactive so we assume that
    /// the caller's desired input and output tokens match the first order they
    /// pass in.
    /// Test a mismatch in the input tokens decimals.
    function testTokenMismatchInputs(
        OrderV3 memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        OrderV3 memory b,
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

        // Line up the tokens so we don't error there.
        b.validInputs[bInputIOIndex].token = a.validInputs[aInputIOIndex].token;
        b.validOutputs[bOutputIOIndex].token = a.validOutputs[aOutputIOIndex].token;

        // Mismatch on inputs decimals across orders taken.
        vm.assume(a.validInputs[aInputIOIndex].decimals != b.validInputs[bInputIOIndex].decimals);
        // Line up output decimals so we don't trigger that code path.
        b.validOutputs[bOutputIOIndex].decimals = a.validOutputs[aOutputIOIndex].decimals;

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](2);
        orders[0] = TakeOrderConfigV3(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV3(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, "");
        vm.expectRevert(
            abi.encodeWithSelector(
                TokenDecimalsMismatch.selector,
                b.validInputs[bInputIOIndex].decimals,
                a.validInputs[aInputIOIndex].decimals
            )
        );
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// Test a mismatch in the output tokens decimals.
    function testTokenMismatchOutputs(
        OrderV2 memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        OrderV2 memory b,
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

        // Line up the tokens so we don't error there.
        b.validInputs[bInputIOIndex].token = a.validInputs[aInputIOIndex].token;
        b.validOutputs[bOutputIOIndex].token = a.validOutputs[aOutputIOIndex].token;

        // Mismatch on outputs decimals across orders taken.
        vm.assume(a.validOutputs[aOutputIOIndex].decimals != b.validOutputs[bOutputIOIndex].decimals);
        // Line up input decimals so we don't trigger that code path.
        b.validInputs[bInputIOIndex].decimals = a.validInputs[aInputIOIndex].decimals;

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](2);
        orders[0] = TakeOrderConfigV3(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV3(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, "");
        vm.expectRevert(
            abi.encodeWithSelector(
                TokenDecimalsMismatch.selector,
                b.validOutputs[bOutputIOIndex].decimals,
                a.validOutputs[aOutputIOIndex].decimals
            )
        );
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }
}
