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
    EvaluableV3,
    SignedContextV1,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {SourceIndexOutOfBounds} from "rain.interpreter.interface/error/ErrBytecode.sol";

/// @title OrderBookClearHandleIORevertTest
/// @notice A test harness for testing the OrderBook clear function will run
/// handle IO and revert if it fails.
contract OrderBookClearHandleIORevertTest is OrderBookExternalRealTest {
    function userDeposit(bytes memory rainString, address owner, address inputToken, address outputToken)
        internal
        returns (OrderV3 memory)
    {
        uint256 vaultId = 0;

        OrderConfigV3 memory config;
        IO[] memory validOutputs;
        IO[] memory validInputs;
        {
            validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, 18, vaultId);
            validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, 18, vaultId);
            // Etch with invalid.
            vm.etch(inputToken, hex"fe");
            vm.etch(outputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(inputToken, "", abi.encode(true));
            vm.mockCall(outputToken, "", abi.encode(true));
        }

        vm.prank(owner);
        iOrderbook.deposit2(outputToken, vaultId, type(uint256).max, new TaskV1[](0));
        assertEq(iOrderbook.vaultBalance(owner, outputToken, vaultId), type(uint256).max);

        bytes memory bytecode = iParserV2.parse2(rainString);
        EvaluableV3 memory evaluable = EvaluableV3(iInterpreter, iStore, bytecode);
        config = OrderConfigV3(evaluable, validInputs, validOutputs, bytes32(0), bytes32(0), "");

        vm.prank(owner);
        vm.recordLogs();
        iOrderbook.addOrder2(config, new TaskV1[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 1);
        (,, OrderV3 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV3));

        return order;
    }

    function checkClearOrderHandleIO(
        bytes memory aliceString,
        bytes memory bobString,
        bytes memory aliceErr,
        bytes memory bobErr
    ) internal {
        address aliceInputToken = address(0x100);
        address aliceOutputToken = address(0x101);
        address alice = address(0x102);
        address bob = address(0x103);

        OrderV3 memory aliceOrder = userDeposit(aliceString, alice, aliceInputToken, aliceOutputToken);
        OrderV3 memory bobOrder = userDeposit(bobString, bob, aliceOutputToken, aliceInputToken);
        ClearConfig memory clearConfig = ClearConfig(0, 0, 0, 0, 0, 0);
        if (aliceErr.length > 0) {
            vm.expectRevert(aliceErr);
        }
        iOrderbook.clear2(aliceOrder, bobOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));

        if (bobErr.length > 0) {
            vm.expectRevert(bobErr);
        }
        iOrderbook.clear2(bobOrder, aliceOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));
    }

    function testClearOrderHandleIO0() external {
        bytes memory aliceString = "_ _:max-value() 1;:ensure(0 \"alice err\");";
        bytes memory bobString = "_ _:max-value() 1;:ensure(0 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, "alice err", "bob err");
    }

    function testClearOrderHandleIO1() external {
        bytes memory aliceString = "_ _:max-value() 1;:;";
        bytes memory bobString = "_ _:max-value() 1;:ensure(0 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, "bob err", "bob err");
    }

    function testClearOrderHandleIO2() external {
        bytes memory aliceString = "_ _:max-value() 1;:ensure(0 \"alice err\");";
        bytes memory bobString = "_ _:max-value() 1;:;";

        checkClearOrderHandleIO(aliceString, bobString, "alice err", "alice err");
    }

    function testClearOrderHandleIO3() external {
        bytes memory aliceString = "_ _:max-value() 1;:ensure(0 \"alice err\");";
        bytes memory bobString = "_ _:max-value() 1;:ensure(0 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, "alice err", "bob err");
    }

    function testClearOrderHandleIO4() external {
        bytes memory aliceErr = "";
        bytes memory bobErr = "";

        bytes memory aliceString = "_ _:max-value() 1;:ensure(1 \"alice err\");";
        bytes memory bobString = "_ _:max-value() 1;:ensure(1 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }

    function testClearOrderHandleIO5() external {
        bytes memory aliceErr = "";
        bytes memory bobErr = "";

        bytes memory aliceString = "_ _:max-value() 1;:;";
        bytes memory bobString = "_ _:max-value() 1;:;";

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }

    /// Note that this error comes from the i9r so it is possible for a different
    /// i9r to not have this error.
    function testClearOrderAliceNoHandleIORevert() external {
        bytes memory aliceString = "_ _:max-value() 1;";
        bytes memory bobString = "_ _:max-value() 1;:;";

        // This is a bit fragile but the error message includes the inner
        // executable bytecode only, not the outer parsed bytecode.
        bytes memory aliceErr =
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200021810000001100000");
        bytes memory bobErr =
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200021810000001100000");

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }

    /// Note that this error comes from the i9r so it is possible for a different
    /// i9r to not have this error.
    function testClearOrderBobNoHandleIORevert() external {
        bytes memory aliceString = "_ _:max-value() 1;:;";
        bytes memory bobString = "_ _:max-value() 1;";

        // This is a bit fragile but the error message includes the inner
        // executable bytecode only, not the outer parsed bytecode.
        bytes memory aliceErr =
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200021810000001100000");
        bytes memory bobErr =
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200021810000001100000");

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }

    /// Note that this error comes from the i9r so it is possible for a different
    /// i9r to not have this error.
    function testClearOrderBothNoHandleIORevert() external {
        bytes memory aliceString = "_ _:max-value() 1;";
        bytes memory bobString = "_ _:max-value() 1;";

        // This is a bit fragile but the error message includes the inner
        // executable bytecode only, not the outer parsed bytecode.
        bytes memory aliceErr =
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200021810000001100000");
        bytes memory bobErr =
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200021810000001100000");

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }
}
