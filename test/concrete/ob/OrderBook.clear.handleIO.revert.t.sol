// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    ClearConfig,
    OrderV2,
    TakeOrderConfigV2,
    IO,
    OrderConfigV2
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";
import {SignedContextV1, EvaluableConfigV3} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";
import {IParserV1} from "rain.interpreter.interface/interface/IParserV1.sol";

/// @title OrderBookClearHandleIORevertTest
/// @notice A test harness for testing the OrderBook clear function will run
/// handle IO and revert if it fails.
contract OrderBookClearHandleIORevertTest is OrderBookExternalRealTest {
    function userDeposit(bytes memory rainString, address owner, address inputToken, address outputToken)
        internal
        returns (OrderV2 memory)
    {
        uint256 vaultId = 0;

        OrderConfigV2 memory config;
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
        iOrderbook.deposit(outputToken, vaultId, type(uint256).max);
        assertEq(iOrderbook.vaultBalance(owner, outputToken, vaultId), type(uint256).max);

        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse(rainString);
        EvaluableConfigV3 memory evaluableConfig = EvaluableConfigV3(iDeployer, bytecode, constants);
        config = OrderConfigV2(validInputs, validOutputs, evaluableConfig, "");

        vm.prank(owner);
        vm.recordLogs();
        iOrderbook.addOrder(config);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        assertEq(entries.length, 3);
        (,, OrderV2 memory order,) = abi.decode(entries[2].data, (address, address, OrderV2, bytes32));

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

        OrderV2 memory aliceOrder = userDeposit(aliceString, alice, aliceInputToken, aliceOutputToken);
        OrderV2 memory bobOrder = userDeposit(bobString, bob, aliceOutputToken, aliceInputToken);
        ClearConfig memory clearConfig = ClearConfig(0, 0, 0, 0, 0, 0);
        if (aliceErr.length > 0) {
            vm.expectRevert(aliceErr);
        }
        iOrderbook.clear(aliceOrder, bobOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));

        if (bobErr.length > 0) {
            vm.expectRevert(bobErr);
        }
        iOrderbook.clear(bobOrder, aliceOrder, clearConfig, new SignedContextV1[](0), new SignedContextV1[](0));
    }

    function testClearOrderHandleIO0() external {
        bytes memory aliceString = "_ _:max-int-value() 1e18;:ensure(0 \"alice err\");";
        bytes memory bobString = "_ _:max-int-value() 1e18;:ensure(0 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, "alice err", "bob err");
    }

    function testClearOrderHandleIO1() external {
        bytes memory aliceString = "_ _:max-int-value() 1e18;:;";
        bytes memory bobString = "_ _:max-int-value() 1e18;:ensure(0 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, "bob err", "bob err");
    }

    function testClearOrderHandleIO2() external {
        bytes memory aliceString = "_ _:max-int-value() 1e18;:ensure(0 \"alice err\");";
        bytes memory bobString = "_ _:max-int-value() 1e18;:;";

        checkClearOrderHandleIO(aliceString, bobString, "alice err", "alice err");
    }

    function testClearOrderHandleIO3() external {
        bytes memory aliceString = "_ _:max-int-value() 1e18;:ensure(0 \"alice err\");";
        bytes memory bobString = "_ _:max-int-value() 1e18;:ensure(0 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, "alice err", "bob err");
    }

    function testClearOrderHandleIO4() external {
        bytes memory aliceErr = "";
        bytes memory bobErr = "";

        bytes memory aliceString = "_ _:max-int-value() 1e18;:ensure(1 \"alice err\");";
        bytes memory bobString = "_ _:max-int-value() 1e18;:ensure(1 \"bob err\");";

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }

    function testClearOrderHandleIO5() external {
        bytes memory aliceErr = "";
        bytes memory bobErr = "";

        bytes memory aliceString = "_ _:max-int-value() 1e18;:;";
        bytes memory bobString = "_ _:max-int-value() 1e18;:;";

        checkClearOrderHandleIO(aliceString, bobString, aliceErr, bobErr);
    }
}
