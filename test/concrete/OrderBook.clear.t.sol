// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/forge-std/src/Test.sol";

// import "test/util/abstract/OrderBookExternalRealTest.sol";
import "test/util/abstract/OrderBookExternalMockTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
    //
    function testClearSimple(
        address alice,
        OrderConfigV2 memory aliceConfig,
        address aliceExpression,
        address bob,
        OrderConfigV2 memory bobConfig,
        address bobExpression
    ) public {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);

        // // Reducing length to one (1) in both ordes (at the moment)
        aliceConfig.validInputs = helperBuildIO(aliceConfig.validInputs);
        aliceConfig.validOutputs = helperBuildIO(aliceConfig.validOutputs);

        // // Bob will have the valid IO swapped.
        bobConfig.validInputs = aliceConfig.validOutputs;
        bobConfig.validOutputs = aliceConfig.validInputs;
        
        // Both index will be zero (since their valid inputs are reduced)
        uint256 inputIOIndex = 0;
        uint256 outputIOIndex = 0;

        console.log("Here_1");

        // 

        vm.prank(alice);
        (, bytes32 aliceOrderhash) = addOrderMockInternal(alice, aliceConfig, aliceExpression);
        vm.prank(bob);
        (, bytes32 bobOrderhash) = addOrderMockInternal(bob, bobConfig, bobExpression);

        // assertTrue(aliceOrderhash != bobOrderhash);

        Vm.Log[] memory logs = vm.getRecordedLogs();
        console.log("Here_2");

        console.log("OrderA:");
        console.logBytes32(aliceOrderhash);
        console.log("OrderB:");
        console.logBytes32(bobOrderhash);

    //   (Order memory aliceOrder, bytes32 aliceOrderhash) = addOrderMockInternal(alice, aliceConfig, aliceExpression);
    //   (Order memory bobOrder, bytes32 bobOrderhash) = addOrderMockInternal(bob, bobConfig, bobExpression);
    }

    // Reduce
    function helperBuildIO(IO[] memory io) pure internal returns (IO[] memory) {
        IO[] memory ioAux = new IO[](1);
        ioAux[0] = io[0];

        return ioAux;
    }

    function addOrderMockInternal(
        address owner,
        OrderConfigV2 memory config,
        address expression
    ) internal returns (Order memory, bytes32) {
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.meta = new bytes(0);

        return addOrderWithChecks(owner, config, expression);
    }

}
