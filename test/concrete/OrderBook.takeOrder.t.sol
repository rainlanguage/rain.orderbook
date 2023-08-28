// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "test/util/abstract/OrderBookExternalRealTest.sol";

/// @title OrderBookTakeOrderTest
/// @notice A test harness for testing the OrderBook takeOrder function.
contract OrderBookTakeOrderTest is OrderBookExternalRealTest {
    function testPrecision01() public {
        uint256 vaultId = 0;
        address inputToken = address(0x100);
        address outputToken = address(0x101);
        OrderConfigV2 memory config;
        {
            uint8 inputTokenDecimals = 6;
            uint8 outputTokenDecimals = 18;
            IO[] memory validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, inputTokenDecimals, vaultId);
            IO[] memory validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, outputTokenDecimals, vaultId);
            // These numbers are known to cause large rounding errors if the
            // precision is not handled correctly.
            (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iDeployer)).parse("output-max io-ratio:157116365680491867129910 318235466963885;:;");
            EvaluableConfigV2 memory evaluableConfig = EvaluableConfigV2(
                iDeployer,
                bytecode,
                constants
            );
            config = OrderConfigV2(
                validInputs,
                validOutputs,
                evaluableConfig,
                ""
            );
            // Etch with invalid.
            vm.etch(outputToken, hex"fe");
            vm.etch(inputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(outputToken, "", abi.encode(true));
            vm.mockCall(inputToken, "", abi.encode(true));
        }
        iOrderbook.deposit(outputToken, vaultId, 1000000e18);
        vm.recordLogs();
        iOrderbook.addOrder(config);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 3);
        (,,Order memory order,) = abi.decode(entries[2].data, (address, address, Order, bytes32));

        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        orders[0] = TakeOrderConfig(
            order,
            0,
            0,
            new SignedContextV1[](0)
        );
        TakeOrdersConfigV2 memory takeOrdersConfig = TakeOrdersConfigV2(
            inputToken,
            outputToken,
            0,
            type(uint256).max,
            type(uint256).max,
            orders,
            ""
        );
        (uint256 totalInput, uint256 totalOutput) = iOrderbook.takeOrders(takeOrdersConfig);
        assertEq(totalInput, 157116365680491867129910);
        assertEq(totalOutput, 50000000);
    }
}