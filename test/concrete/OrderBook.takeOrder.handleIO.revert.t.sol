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
    function checkTakeOrderHandleIO(bytes[] memory configs, bytes memory err, uint256 maxInput) internal {
        uint256 vaultId = 0;
        address inputToken = address(0x100);
        address outputToken = address(0x101);

        OrderConfigV2 memory config;
        IO[] memory validOutputs;
        IO[] memory validInputs;
        {
            validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, 18, vaultId);
            validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, 18, vaultId);
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

        TakeOrderConfig[] memory orders = new TakeOrderConfig[](configs.length);

        for (uint256 i = 0; i < configs.length; i++) {
            (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iDeployer)).parse(configs[i]);
            EvaluableConfigV2 memory evaluableConfig = EvaluableConfigV2(iDeployer, bytecode, constants);
            config = OrderConfigV2(validInputs, validOutputs, evaluableConfig, "");

            vm.recordLogs();
            iOrderbook.addOrder(config);
            Vm.Log[] memory entries = vm.getRecordedLogs();
            assertEq(entries.length, 3);
            (,, Order memory order,) = abi.decode(entries[2].data, (address, address, Order, bytes32));

            orders[i] = TakeOrderConfig(order, 0, 0, new SignedContextV1[](0));
        }
        TakeOrdersConfigV2 memory takeOrdersConfig = TakeOrdersConfigV2(0, maxInput, type(uint256).max, orders, "");

        if (err.length > 0) {
            vm.expectRevert(err);
        }
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(takeOrdersConfig);
        // We don't really care about the outputs as the tests are basically just
        // trying to show that the IO handler is running or not running by simple
        // reverts.
        (totalTakerInput, totalTakerOutput);
    }

    function testTakeOrderHandleIO0() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 1, 0);
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:max-int-value() 1e18;:ensure<1>(0);";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO1() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 1, 0);
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1e18 1e18;:ensure<1>(0);";
        configs[1] = "_ _:1e18 1e18;:;";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO2() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 1, 0);
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure<1>(0);";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO3() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 1, 0);
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure<1>(0);";
        configs[2] = "_ _:1e18 1e18;:;";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO4() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 1, 0);
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure<1>(0);";
        configs[2] = "_ _:1e18 1e18;:ensure<2>(0);";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO5() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 2, 0);
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure<2>(0);";
        configs[2] = "_ _:1e18 1e18;:ensure<1>(0);";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO6() external {
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 2, 0);
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:ensure<2>(0);";
        configs[1] = "_ _:1e18 1e18;:;";
        configs[2] = "_ _:1e18 1e18;:ensure<1>(0);";
        checkTakeOrderHandleIO(configs, err, type(uint256).max);
    }

    function testTakeOrderHandleIO7(uint256 toClear) external {
        toClear = bound(toClear, 3e18 + 1, type(uint256).max);
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 2, 0);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1e18 1e18;:set(0 1);";
        configs[1] = "_ _:1e18 1e18;:ensure<1>(get(0));";
        configs[2] = "_ _:1e18 1e18;:set(0 0);";
        configs[3] = "_ _:1e18 1e18;:ensure<2>(get(0));";
        checkTakeOrderHandleIO(configs, err, toClear);
    }

    function testTakeOrderHandleIO8(uint256 toClear) external {
        toClear = bound(toClear, 4e18 + 1, type(uint256).max);
        bytes memory err = abi.encodeWithSelector(EnsureFailed.selector, 2, 0);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:set(0 1);";
        configs[2] = "_ _:1e18 1e18;:ensure<1>(get(0));";
        configs[3] = "_ _:1e18 1e18;:set(0 0);";
        configs[4] = "_ _:1e18 1e18;:ensure<2>(get(0));";
        checkTakeOrderHandleIO(configs, err, toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    function testTakeOrderHandleIO9(uint256 toClear) external {
        toClear = bound(toClear, 1, 4e18);
        bytes memory err = "";
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:set(0 1);";
        configs[2] = "_ _:1e18 1e18;:ensure<1>(get(0));";
        configs[3] = "_ _:1e18 1e18;:set(0 0);";
        configs[4] = "_ _:1e18 1e18;:ensure<2>(get(0));";
        checkTakeOrderHandleIO(configs, err, toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    function testTakeOrderHandleIO10(uint256 toClear) external {
        toClear = bound(toClear, 1, 3e18);
        bytes memory err = "";
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1e18 1e18;:set(0 1);";
        configs[1] = "_ _:1e18 1e18;:ensure<1>(get(0));";
        configs[2] = "_ _:1e18 1e18;:set(0 0);";
        configs[3] = "_ _:1e18 1e18;:ensure<2>(get(0));";
        checkTakeOrderHandleIO(configs, err, toClear);
    }
}
