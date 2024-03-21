// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {OrderConfigV2} from "rain.orderbook.interface/interface/unstable/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter.interface/interface/IParserV1.sol";
import {
    UnsupportedCalculateOutputs,
    UnsupportedCalculateInputs,
    UnsupportedHandleInputs
} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookAddOrderTest
/// @notice A test harness for testing the OrderBook addOrder function.
contract OrderBookAddOrderTest is OrderBookExternalRealTest {
    /// No sources reverts as we need at least a calculate expression.
    function testAddOrderRealNoSourcesReverts(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        config.evaluableConfig.bytecode = hex"";
        vm.expectRevert(abi.encodeWithSelector(OrderNoSources.selector));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// No handle IO reverts.
    function testAddOrderRealNoHandleIOReverts(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse(":;");
        config.evaluableConfig.bytecode = bytecode;
        config.evaluableConfig.constants = constants;
        vm.expectRevert(abi.encodeWithSelector(OrderNoHandleIO.selector));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 0 for calculate order reverts.
    function testAddOrderRealZeroStackCalculateReverts(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse(":;:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, 0));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 1 for calculate order reverts.
    function testAddOrderRealOneStackCalculateReverts(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iParser)).parse("_:block-timestamp();:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, 1));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 2 for calculate order deploys.
    function testAddOrderRealTwoStackCalculateReverts(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iParser)).parse("_ _:block-timestamp() chain-id();:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 3 for calculate order deploys.
    function testAddOrderRealThreeStackCalculate(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iParser)).parse("_ _ _:block-timestamp() chain-id() block-number();:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// Inputs for calculate order errors. Tests one input.
    function testAddOrderRealCalculateInputsReverts1(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse("i:;:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateInputs.selector, 1));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// Inputs for calculate order errors. Tests two inputs.
    function testAddOrderRealCalculateInputsReverts2(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse("i0 i1:;:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateInputs.selector, 2));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// Inputs for calculate order errors. This takes precedent over the same
    /// error in handle io.
    function testAddOrderRealCalculateInputsRevertsPreference(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse("i:;i:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateInputs.selector, 1));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// Inputs for handle io errors. Tests one input.
    function testAddOrderRealHandleIOInputsReverts1(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse("_ _:1e18 1e18;i:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedHandleInputs.selector, 1));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// Inputs for handle io errors. Tests two inputs.
    function testAddOrderRealHandleIOInputsReverts2(address owner, OrderConfigV2 memory config) public {
        LibTestAddOrder.conformConfig(config, iDeployer);
        (bytes memory bytecode, uint256[] memory constants) = IParserV1(address(iParser)).parse("_ _:1e18 1e18;i0 i1:;");
        config.evaluableConfig.constants = constants;
        config.evaluableConfig.bytecode = bytecode;
        vm.expectRevert(abi.encodeWithSelector(UnsupportedHandleInputs.selector, 2));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }
}
