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
    function userDeposit(bytes memory rainString, address owner, address inputToken, address outputToken, uint8 inputDecimals, uint8 outputDecimals)
        internal
        returns (OrderV3 memory)
    {
        uint256 vaultId = 0;

        OrderConfigV3 memory config;
        IO[] memory validOutputs;
        IO[] memory validInputs;
        {
            validInputs = new IO[](1);
            validInputs[0] = IO(inputToken, inputDecimals, vaultId);
            validOutputs = new IO[](1);
            validOutputs[0] = IO(outputToken, outputDecimals, vaultId);
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

    function testClearOrderHandleRoundingIO1() external {

        address aliceInputToken = address(0x100);
        address aliceOutputToken = address(0x101);
        address alice = address(0x102);
        address bob = address(0x103);

        bytes memory aliceString = "_ _:6.033684273070069670 0.003033612425676495;";
        bytes memory bobString = "_ _:0.252855759812926844 253.583614038823650697;";

        OrderV3 memory aliceOrder = userDeposit(aliceString, alice, aliceInputToken, aliceOutputToken, 18, 6);
        OrderV3 memory bobOrder = userDeposit(bobString, bob, aliceOutputToken, aliceInputToken, 6, 18);

        ClearConfig memory clearConfig = ClearConfig(0, 0, 0, 0, 1, 1);
        iOrderbook.clear2(aliceOrder, bobOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));
        
    }



}
