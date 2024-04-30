// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Vm} from "forge-std/Test.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderV2,
    TakeOrdersConfigV2,
    TakeOrderConfigV2,
    IO,
    OrderConfigV2
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter.interface/interface/IParserV1.sol";
import {SignedContextV1, EvaluableConfigV3} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";

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
        OrderConfigV2 memory config;
        {
            IO[] memory validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, inputTokenDecimals, vaultId);
            IO[] memory validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, outputTokenDecimals, vaultId);
            // These numbers are known to cause large rounding errors if the
            // precision is not handled correctly.
            (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse(rainString);
            EvaluableConfigV3 memory evaluableConfig = EvaluableConfigV3(iDeployer, bytecode, constants);
            config = OrderConfigV2(validInputs, validOutputs, evaluableConfig, "");
            // Etch with invalid.
            vm.etch(outputToken, hex"fe");
            vm.etch(inputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(outputToken, "", abi.encode(true));
            vm.mockCall(inputToken, "", abi.encode(true));
        }
        if (expectedTakerTotalInput > 0) {
            iOrderbook.deposit(outputToken, vaultId, expectedTakerTotalInput);
        }
        assertEq(iOrderbook.vaultBalance(address(this), outputToken, vaultId), expectedTakerTotalInput);
        vm.recordLogs();
        iOrderbook.addOrder(config);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 3);
        (,, OrderV2 memory order,) = abi.decode(entries[2].data, (address, address, OrderV2, bytes32));

        TakeOrderConfigV2[] memory orders = new TakeOrderConfigV2[](1);
        orders[0] = TakeOrderConfigV2(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory takeOrdersConfig =
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, "");
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(takeOrdersConfig);
        assertEq(totalTakerInput, expectedTakerTotalInput);
        assertEq(totalTakerOutput, expectedTakerTotalOutput);
        assertEq(iOrderbook.vaultBalance(address(this), outputToken, vaultId), 0);
    }

    function testTakeOrderPrecisionKnownBad01() public {
        // Older versions of OB had precision issues with this IO setup.
        bytes memory knownBad = "output-max io-ratio:157116365680491867129910 318235466963885;:;";
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
