// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    ClearConfig,
    OrderV2,
    TakeOrderConfigV2,
    IO,
    OrderConfigV2,
    TakeOrdersConfigV2
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter.interface/interface/IParserV1.sol";
import {SignedContextV1, EvaluableConfigV3} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";

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

        TakeOrderConfigV2[] memory orders = new TakeOrderConfigV2[](configs.length);

        for (uint256 i = 0; i < configs.length; i++) {
            (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse(configs[i]);
            EvaluableConfigV3 memory evaluableConfig = EvaluableConfigV3(iDeployer, bytecode, constants);
            config = OrderConfigV2(validInputs, validOutputs, evaluableConfig, "");

            vm.recordLogs();
            iOrderbook.addOrder(config);
            Vm.Log[] memory entries = vm.getRecordedLogs();
            assertEq(entries.length, 3);
            (,, OrderV2 memory order,) = abi.decode(entries[2].data, (address, address, OrderV2, bytes32));

            orders[i] = TakeOrderConfigV2(order, 0, 0, new SignedContextV1[](0));
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
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:max-int-value() 1e18;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO1() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1e18 1e18;:ensure(0 \"err\");";
        configs[1] = "_ _:1e18 1e18;:;";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO2() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO3() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure(0 \"err\");";
        configs[2] = "_ _:1e18 1e18;:;";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO4() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure(0 \"err 1\");";
        configs[2] = "_ _:1e18 1e18;:ensure(0 \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 1", type(uint256).max);
    }

    function testTakeOrderHandleIO5() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:ensure(0 \"err 2\");";
        configs[2] = "_ _:1e18 1e18;:ensure(0 \"err 1\");";
        checkTakeOrderHandleIO(configs, "err 2", type(uint256).max);
    }

    function testTakeOrderHandleIO6() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1e18 1e18;:ensure(0 \"err 2\");";
        configs[1] = "_ _:1e18 1e18;:;";
        configs[2] = "_ _:1e18 1e18;:ensure(0 \"err 1\");";
        checkTakeOrderHandleIO(configs, "err 2", type(uint256).max);
    }

    function testTakeOrderHandleIO7(uint256 toClear) external {
        toClear = bound(toClear, 3e18 + 1, type(uint256).max);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1e18 1e18;:set(0 1);";
        configs[1] = "_ _:1e18 1e18;:ensure(get(0) \"err 1\");";
        configs[2] = "_ _:1e18 1e18;:set(0 0);";
        configs[3] = "_ _:1e18 1e18;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 2", toClear);
    }

    function testTakeOrderHandleIO8(uint256 toClear) external {
        toClear = bound(toClear, 4e18 + 1, type(uint256).max);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:set(0 1);";
        configs[2] = "_ _:1e18 1e18;:ensure(get(0) \"err 1\");";
        configs[3] = "_ _:1e18 1e18;:set(0 0);";
        configs[4] = "_ _:1e18 1e18;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 2", toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    function testTakeOrderHandleIO9(uint256 toClear) external {
        toClear = bound(toClear, 1, 4e18);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1e18 1e18;:;";
        configs[1] = "_ _:1e18 1e18;:set(0 1);";
        configs[2] = "_ _:1e18 1e18;:ensure(get(0) \"err 1\");";
        configs[3] = "_ _:1e18 1e18;:set(0 0);";
        configs[4] = "_ _:1e18 1e18;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "", toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    function testTakeOrderHandleIO10(uint256 toClear) external {
        toClear = bound(toClear, 1, 3e18);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1e18 1e18;:set(0 1);";
        configs[1] = "_ _:1e18 1e18;:ensure(get(0) \"err 1\");";
        configs[2] = "_ _:1e18 1e18;:set(0 0);";
        configs[3] = "_ _:1e18 1e18;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "", toClear);
    }
}
