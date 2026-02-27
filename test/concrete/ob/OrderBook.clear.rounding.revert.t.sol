// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {TOFU_DECIMALS_SELECTOR} from "../../../src/lib/LibTOFUTokenDecimals.sol";
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

    function userDeposit(
        bytes memory rainString,
        address owner,
        address inputToken,
        address outputToken,
        uint8 inputDecimals,
        uint8 outputDecimals,
        int256 outputDeposit
    ) internal returns (OrderV4 memory) {
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
            // Mock decimals() calls to return the specified decimals
            vm.mockCall(inputToken, TOFU_DECIMALS_SELECTOR, abi.encode(inputDecimals));
            vm.mockCall(outputToken, TOFU_DECIMALS_SELECTOR, abi.encode(outputDecimals));
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(inputToken, bytes(""), abi.encode(true));
            vm.mockCall(outputToken, bytes(""), abi.encode(true));
        }

        vm.prank(owner);
        iOrderbook.deposit3(outputToken, vaultId, LibDecimalFloat.packLossless(outputDeposit, 0), new TaskV2[](0));
        Float balance = iOrderbook.vaultBalance2(owner, outputToken, vaultId);
        assertTrue(balance.eq(LibDecimalFloat.packLossless(outputDeposit, 0)));

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

    function testClearOrderHandleRoundingIO() external {
        address aliceInputToken = address(0x100);
        address aliceOutputToken = address(0x101);
        address alice = address(0x102);
        address bob = address(0x103);
        address carol = address(0x104);

        bytes memory aliceString = "_ _:6.033684273070069670 0.003033612425676495;:;";
        bytes memory bobString = "_ _:0.252855759812926844 253.583614038823650697;:;";

        OrderV4 memory aliceOrder = userDeposit(aliceString, alice, aliceInputToken, aliceOutputToken, 18, 6, 1000e6);
        OrderV4 memory bobOrder = userDeposit(bobString, bob, aliceOutputToken, aliceInputToken, 6, 18, 1000e18);

        ClearConfigV2 memory clearConfig = ClearConfigV2(0, 0, 0, 0, 0, 0);

        vm.startPrank(carol);
        iOrderbook.clear3(aliceOrder, bobOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));
        vm.stopPrank();
    }
}
