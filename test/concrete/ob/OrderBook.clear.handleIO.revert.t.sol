// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    ClearConfigV2,
    OrderV4,
    TakeOrderConfigV4,
    IOV2,
    OrderConfigV4,
    EvaluableV4,
    SignedContextV1,
    TaskV2,
    Float
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {SourceIndexOutOfBounds} from "rain.interpreter.interface/error/ErrBytecode.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title OrderBookClearHandleIORevertTest
/// @notice A test harness for testing the OrderBook clear function will run
/// handle IO and revert if it fails.
contract OrderBookClearHandleIORevertTest is OrderBookExternalRealTest {
    using LibDecimalFloat for Float;

    function userDeposit(bytes memory rainString, address owner, address inputToken, address outputToken)
        internal
        returns (OrderV4 memory)
    {
        bytes32 vaultId = 0;

        OrderConfigV4 memory config;
        IOV2[] memory validOutputs;
        IOV2[] memory validInputs;
        {
            validInputs = new IOV2[](1);
            validInputs[0] = IOV2(inputToken, vaultId);
            validOutputs = new IOV2[](1);
            validOutputs[0] = IOV2(outputToken, vaultId);
            // Etch with invalid.
            vm.etch(inputToken, hex"fe");
            vm.etch(outputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(inputToken, "", abi.encode(true));
            vm.mockCall(outputToken, "", abi.encode(true));
        }

        vm.prank(owner);
        iOrderbook.deposit3(outputToken, vaultId, LibDecimalFloat.packLossless(type(int256).max, 0), new TaskV2[](0));
        Float balance = iOrderbook.vaultBalance2(owner, outputToken, vaultId);
        assertTrue(balance.eq(LibDecimalFloat.packLossless(type(int256).max, type(int256).max)));

        bytes memory bytecode = iParserV2.parse2(rainString);
        EvaluableV4 memory evaluable = EvaluableV4(iInterpreter, iStore, bytecode);
        config = OrderConfigV4(evaluable, validInputs, validOutputs, bytes32(0), bytes32(0), "");

        vm.prank(owner);
        vm.recordLogs();
        iOrderbook.addOrder3(config, new TaskV2[](0));
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 1);
        (,, OrderV4 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV4));

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

        OrderV4 memory aliceOrder = userDeposit(aliceString, alice, aliceInputToken, aliceOutputToken);
        OrderV4 memory bobOrder = userDeposit(bobString, bob, aliceOutputToken, aliceInputToken);
        ClearConfigV2 memory clearConfig = ClearConfigV2(0, 0, 0, 0, 0, 0);
        if (aliceErr.length > 0) {
            vm.expectRevert(aliceErr);
        }
        iOrderbook.clear3(aliceOrder, bobOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));

        if (bobErr.length > 0) {
            vm.expectRevert(bobErr);
        }
        iOrderbook.clear3(bobOrder, aliceOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));
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
