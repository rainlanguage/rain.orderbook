// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {Vm} from "forge-std/Test.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderV3,
    TakeOrdersConfigV3,
    TakeOrderConfigV3,
    IO,
    OrderConfigV3,
    EvaluableV3,
    SignedContextV1,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

/// @title OrderBookTakeOrderPrecisionTest
/// @notice A test harness for testing the OrderBook takeOrder function.
contract OrderBookTakeOrderPrecisionTest is OrderBookExternalRealTest {
    function checkPrecision(
        bytes memory rainString,
        uint8 outputTokenDecimals,
        uint8 inputTokenDecimals,
        uint256 expectedTakerTotalInput,
        uint256 expectedTakerTotalOutput
    ) internal {
        uint256 vaultId = 0;
        address inputToken = address(0x100);
        address outputToken = address(0x101);
        OrderConfigV3 memory config;
        {
            IO[] memory validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, inputTokenDecimals, vaultId);
            IO[] memory validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, outputTokenDecimals, vaultId);
            // These numbers are known to cause large rounding errors if the
            // precision is not handled correctly.
            bytes memory bytecode = iParserV2.parse2(rainString);
            EvaluableV3 memory evaluable = EvaluableV3(iInterpreter, iStore, bytecode);
            config = OrderConfigV3(evaluable, validInputs, validOutputs, bytes32(0), bytes32(0), "");
            // Etch with invalid.
            vm.etch(outputToken, hex"fe");
            vm.etch(inputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(outputToken, "", abi.encode(true));
            vm.mockCall(inputToken, "", abi.encode(true));
        }
        if (expectedTakerTotalInput > 0) {
            iOrderbook.deposit2(outputToken, vaultId, expectedTakerTotalInput, new TaskV1[](0));
        }
        assertEq(iOrderbook.vaultBalance(address(this), outputToken, vaultId), expectedTakerTotalInput);
        vm.recordLogs();
        iOrderbook.addOrder2(config, new TaskV1[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 1);
        (,, OrderV3 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV3));

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](1);
        orders[0] = TakeOrderConfigV3(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV3 memory takeOrdersConfig =
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, "");
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders2(takeOrdersConfig);
        assertEq(totalTakerInput, expectedTakerTotalInput);
        assertEq(totalTakerOutput, expectedTakerTotalOutput);
        assertEq(iOrderbook.vaultBalance(address(this), outputToken, vaultId), 0);
    }

    function testTakeOrderPrecisionKnownBad01() public {
        // Older versions of OB had precision issues with this IO setup.
        bytes memory knownBad = "output-max io-ratio:157116365680491867129910e-18 318235466963885e-18;:;";
        // Start with both tokens having 18 decimals.
        // This gives the best precision for both.
        checkPrecision(knownBad, 18, 18, 157116365680491867129910, 49999999999999844580);

        // If the taker output token has low decimals then it will round up
        // at that decimal precision, to force the taker to have to output the
        // dust amount.
        // Increasing the decimals of the taker input token will not impact the
        // precision provided there is no overflow. It simply scales up the
        // taker input amount.
        checkPrecision(knownBad, 18, 6, 157116365680491867129910, 50e6);
        checkPrecision(knownBad, 19, 6, 1571163656804918671299100, 50e6);
        checkPrecision(knownBad, 20, 6, 15711636568049186712991000, 50e6);
        checkPrecision(knownBad, 21, 6, 157116365680491867129910000, 50e6);
        checkPrecision(knownBad, 50, 6, 15711636568049186712991000000000000000000000000000000000, 50e6);

        // Flip the decimals for each token.
        // As the output has low decimals, it rounds down which then causes the
        // input to be slightly smaller prorata.
        checkPrecision(knownBad, 6, 18, 157116365680, 49999999999843315014);
        checkPrecision(knownBad, 6, 19, 157116365680, 499999999998433150140);
        checkPrecision(knownBad, 6, 20, 157116365680, 4999999999984331501400);
        checkPrecision(knownBad, 6, 21, 157116365680, 49999999999843315014000);
        checkPrecision(knownBad, 6, 50, 157116365680, 4999999999984331501400000000000000000000000000000000);
    }
}
