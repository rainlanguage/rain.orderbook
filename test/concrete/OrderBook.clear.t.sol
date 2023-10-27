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
        // Reducing length to this in both ordes (at the moment)
        vm.assume(aliceConfig.validInputs.length == 1);
        vm.assume(aliceConfig.validOutputs.length == 1);
        vm.assume(aliceConfig.validInputs.length == 1);
        vm.assume(aliceConfig.validOutputs.length == 1);
        
        uint256 inputIOIndex = 0;
        uint256 outputIOIndex = 0;


        // The inputs and outpus will match this way. Alice input should match with bob output and viceversa.
        aliceConfig.validInputs[inputIOIndex1].token = bobConfig.validOutputs[inputIOIndex2].token;
        aliceConfig.validInputs[inputIOIndex1].decimals = bobConfig.validOutputs[inputIOIndex2].decimals;
        aliceConfig.validOutputs[outputIOIndex1].token = bobConfig.validInputs[outputIOIndex2].token;
        aliceConfig.validOutputs[outputIOIndex1].decimals = bobConfig.validInputs[outputIOIndex2].decimals;

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
