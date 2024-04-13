// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {IParserV2} from "rain.interpreter.interface/interface/unstable/IParserV2.sol";
import {
    UnsupportedCalculateOutputs,
    UnsupportedCalculateInputs,
    UnsupportedHandleInputs
} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookAddOrderTest
/// @notice A test harness for testing the OrderBook addOrder function.
contract OrderBookAddOrderTest is OrderBookExternalRealTest {
    /// No sources reverts as we need at least a calculate expression.
    function testAddOrderRealNoSourcesReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = hex"";
        vm.expectRevert(abi.encodeWithSelector(OrderNoSources.selector));
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// No handle IO reverts.
    function testAddOrderRealNoHandleIOReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2(":;");
        config.evaluable.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(OrderNoHandleIO.selector));
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// A stack of 0 for calculate order deploys.
    /// Stack size checks are runtime.
    function testAddOrderRealZeroStackCalculateReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2(":;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// A stack of 1 for calculate order reverts.
    /// Stack size checks are runtime.
    function testAddOrderRealOneStackCalculateReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_:block-timestamp();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// A stack of 2 for calculate order deploys.
    function testAddOrderRealTwoStackCalculateReverts(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _:block-timestamp() chain-id();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// A stack of 3 for calculate order deploys.
    function testAddOrderRealThreeStackCalculate(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _ _:block-timestamp() chain-id() block-number();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// Inputs for calculate order. Tests one input.
    /// Deploys because this is a runtime check.
    function testAddOrderRealCalculateInputsReverts1(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i:;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// Inputs for calculate order errors. Tests two inputs.
    /// Deploys because this is a runtime check.
    function testAddOrderRealCalculateInputsReverts2(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i0 i1:;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// Inputs for calculate order errors. This takes precedent over the same
    /// error in handle io.
    /// Deploys because this is a runtime check.
    function testAddOrderRealCalculateInputsRevertsPreference(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i:;i:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// Inputs for handle io errors. Tests one input.
    /// Deploys because this is a runtime check.
    function testAddOrderRealHandleIOInputsReverts1(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _:1e18 1e18;i:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }

    /// Inputs for handle io errors. Tests two inputs.
    /// Deploys because this is a runtime check.
    function testAddOrderRealHandleIOInputsReverts2(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _:1e18 1e18;i0 i1:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder2(config, new EvaluableV3[](0));
    }
}
