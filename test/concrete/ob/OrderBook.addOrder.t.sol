// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {OrderConfigV3, EvaluableV3, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {UnsupportedCalculateOutputs, UnsupportedCalculateInputs} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookAddOrderTest
/// @notice A test harness for testing the OrderBook addOrder function.
contract OrderBookAddOrderTest is OrderBookExternalRealTest {
    /// No sources deploys as we let this be a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealNoSourcesDeploys(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = hex"";
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// No handle IO reverts.
    /// This is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealNoHandleIODeploys(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2(":;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// A stack of 0 for calculate order deploys.
    /// Stack size checks are runtime.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealZeroStackCalculateReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2(":;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// A stack of 1 for calculate order reverts.
    /// Stack size checks are runtime.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealOneStackCalculateReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_:block-timestamp();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// A stack of 2 for calculate order deploys.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealTwoStackCalculateReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _:block-timestamp() chain-id();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// A stack of 3 for calculate order deploys.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealThreeStackCalculate(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _ _:block-timestamp() chain-id() block-number();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// Inputs for calculate order. Tests one input.
    /// Deploys because this is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealCalculateInputsReverts1(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i:;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// Inputs for calculate order errors. Tests two inputs.
    /// Deploys because this is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealCalculateInputsReverts2(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i0 i1:;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }

    /// Inputs for calculate order errors. This takes precedent over the same
    /// error in handle io.
    /// Deploys because this is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealCalculateInputsRevertsPreference(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i:;i:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new TaskV1[](0));
    }
}
