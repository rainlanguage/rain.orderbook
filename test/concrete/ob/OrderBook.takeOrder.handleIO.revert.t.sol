// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    ClearConfig,
    OrderV3,
    TakeOrderConfigV3,
    IO,
    OrderConfigV3,
    TakeOrdersConfigV3,
    EvaluableV3,
    SignedContextV1,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {SourceIndexOutOfBounds} from "rain.interpreter.interface/error/ErrBytecode.sol";

/// @title OrderBookTakeOrderHandleIORevertTest
/// @notice A test harness for testing the OrderBook takeOrder function will run
/// handle IO and revert if it fails.
contract OrderBookTakeOrderHandleIORevertTest is OrderBookExternalRealTest {
    function checkTakeOrderHandleIO(bytes[] memory configs, bytes memory err, uint256 maxInput) internal {
        uint256 vaultId = 0;
        address inputToken = address(0x100);
        address outputToken = address(0x101);

        OrderConfigV3 memory config;
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
        iOrderbook.deposit2(outputToken, vaultId, type(uint256).max, new TaskV2[](0));
        assertEq(iOrderbook.vaultBalance(address(this), outputToken, vaultId), type(uint256).max);

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](configs.length);

        for (uint256 i = 0; i < configs.length; i++) {
            bytes memory bytecode = iParserV2.parse2(configs[i]);
            EvaluableV3 memory evaluable = EvaluableV3(iInterpreter, iStore, bytecode);
            config = OrderConfigV3(evaluable, validInputs, validOutputs, bytes32(i), bytes32(0), "");

            vm.recordLogs();
            iOrderbook.addOrder2(config, new TaskV2[](0));
            Vm.Log[] memory entries = vm.getRecordedLogs();
            assertEq(entries.length, 1);
            (,, OrderV3 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV3));

            orders[i] = TakeOrderConfigV3(order, 0, 0, new SignedContextV1[](0));
        }
        TakeOrdersConfigV3 memory takeOrdersConfig = TakeOrdersConfigV3(0, maxInput, type(uint256).max, orders, "");

        if (err.length > 0) {
            vm.expectRevert(err);
        }
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders2(takeOrdersConfig);
        // We don't really care about the outputs as the tests are basically just
        // trying to show that the IO handler is running or not running by simple
        // reverts.
        (totalTakerInput, totalTakerOutput);
    }

    function testTakeOrderHandleIO0() external {
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:max-value() 1;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO1() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;:ensure(0 \"err\");";
        configs[1] = "_ _:1 1;:;";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO2() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO3() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err\");";
        configs[2] = "_ _:1 1;:;";
        checkTakeOrderHandleIO(configs, "err", type(uint256).max);
    }

    function testTakeOrderHandleIO4() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err 1\");";
        configs[2] = "_ _:1 1;:ensure(0 \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 1", type(uint256).max);
    }

    function testTakeOrderHandleIO5() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err 2\");";
        configs[2] = "_ _:1 1;:ensure(0 \"err 1\");";
        checkTakeOrderHandleIO(configs, "err 2", type(uint256).max);
    }

    function testTakeOrderHandleIO6() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:ensure(0 \"err 2\");";
        configs[1] = "_ _:1 1;:;";
        configs[2] = "_ _:1 1;:ensure(0 \"err 1\");";
        checkTakeOrderHandleIO(configs, "err 2", type(uint256).max);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO7(uint256 toClear) external {
        toClear = bound(toClear, 3e18 + 1, type(uint256).max);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1 1;:set(0 1);";
        configs[1] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[2] = "_ _:1 1;:set(0 0);";
        configs[3] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 2", toClear);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO8(uint256 toClear) external {
        toClear = bound(toClear, 4e18 + 1, type(uint256).max);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:set(0 1);";
        configs[2] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[3] = "_ _:1 1;:set(0 0);";
        configs[4] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 2", toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO9(uint256 toClear) external {
        toClear = bound(toClear, 1, 4e18);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:set(0 1);";
        configs[2] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[3] = "_ _:1 1;:set(0 0);";
        configs[4] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "", toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO10(uint256 toClear) external {
        toClear = bound(toClear, 1, 3e18);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1 1;:set(0 1);";
        configs[1] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[2] = "_ _:1 1;:set(0 0);";
        configs[3] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "", toClear);
    }

    /// Note that a different interpreter MAY NOT revert if handle io is missing,
    /// but the canonical interpreter will.
    function testTakeOrderNoHandleIORevert0() external {
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:1 1;";
        checkTakeOrderHandleIO(
            configs,
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200020110000001100000"),
            type(uint256).max
        );
    }

    /// Note that a different interpreter MAY NOT revert if handle io is missing,
    /// but the canonical interpreter will.
    function testTakeOrderNoHandleIORevert1() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;";
        checkTakeOrderHandleIO(
            configs,
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200020110000001100000"),
            type(uint256).max
        );
    }

    /// Note that a different interpreter MAY NOT revert if handle io is missing,
    /// but the canonical interpreter will.
    function testTakeOrderNoHandleIORevert2() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;";
        configs[1] = "_ _:1 1;:;";
        checkTakeOrderHandleIO(
            configs,
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200020110000001100000"),
            type(uint256).max
        );
    }
}
