// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {
    OrderBookExternalRealTest,
    OrderConfigV2,
    IO,
    IParserV1,
    EvaluableConfigV2,
    Order,
    TakeOrderConfig,
    SignedContextV1,
    TakeOrdersConfigV2
} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {EnsureFailed} from "rain.interpreter/src/lib/op/logic/LibOpEnsureNP.sol";

/// @title OrderBookTakeOrderHandleIORevertTest
/// @notice A test harness for testing the OrderBook takeOrder function will run
/// handle IO and revert if it fails.
contract OrderBookTakeOrderHandleIORevertTest is OrderBookExternalRealTest {
    function testTakeOrderHandleIORevert() public {
        uint256 vaultId = 0;
        address inputToken = address(0x100);
        address outputToken = address(0x101);
        OrderConfigV2 memory config;
        {
            IO[] memory validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, 18, vaultId);
            IO[] memory validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, 18, vaultId);
            (bytes memory bytecode, uint256[] memory constants) =
                IParserV1(address(iDeployer)).parse("_ _:max-int-value() 1e18;:ensure<1>(0);");
            EvaluableConfigV2 memory evaluableConfig = EvaluableConfigV2(iDeployer, bytecode, constants);
            config = OrderConfigV2(validInputs, validOutputs, evaluableConfig, "");
            // Etch with invalid.
            vm.etch(outputToken, hex"fe");
            vm.etch(inputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(outputToken, "", abi.encode(true));
            vm.mockCall(inputToken, "", abi.encode(true));
        }
        iOrderbook.deposit(outputToken, vaultId, type(uint256).max);
        assertEq(iOrderbook.vaultBalance(address(this), outputToken, vaultId), type(uint256).max);

        vm.recordLogs();
        iOrderbook.addOrder(config);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 3);
        (,, Order memory order,) = abi.decode(entries[2].data, (address, address, Order, bytes32));

        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        orders[0] = TakeOrderConfig(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV2 memory takeOrdersConfig =
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, "");

        vm.expectRevert(abi.encodeWithSelector(EnsureFailed.selector, 1, 0));
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(takeOrdersConfig);
        (totalTakerInput, totalTakerOutput);
    }
}
