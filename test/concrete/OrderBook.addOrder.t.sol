// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "test/util/abstract/OrderBookExternalRealTest.sol";
import "test/util/lib/LibTestAddOrder.sol";

/// @title OrderBookAddOrderTest
/// @notice A test harness for testing the OrderBook addOrder function.
contract OrderBookAddOrderTest is OrderBookExternalRealTest {
    /// No sources reverts as we need at least a calculate expression.
    function testAddOrderRealNoSourcesReverts(address owner, OrderConfig memory config) public {
        vm.assume(LibTestAddOrder.conformConfig(config, iDeployer));
        config.evaluableConfig.sources = new bytes[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoSources.selector, owner));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// No handle IO reverts.
    function testAddOrderRealNoHandleIOReverts(address owner, OrderConfig memory config) public {
        vm.assume(LibTestAddOrder.conformConfig(config, iDeployer));
        (bytes[] memory sources, uint256[] memory constants) = IParserV1(address(iDeployer)).parse(":;");
        (constants);
        config.evaluableConfig.sources = sources;
        vm.expectRevert(abi.encodeWithSelector(OrderNoHandleIO.selector, owner));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 0 for calculate order reverts.
    function testAddOrderRealZeroStackCalculateReverts(address owner, OrderConfig memory config) public {
        vm.assume(LibTestAddOrder.conformConfig(config, iDeployer));
        (bytes[] memory sources, uint256[] memory constants) = IParserV1(address(iDeployer)).parse(":;:;");
        (constants);
        config.evaluableConfig.sources = sources;
        vm.expectRevert(abi.encodeWithSelector(MinFinalStack.selector, 2, 0));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 1 for calculate order reverts.
    function testAddOrderRealOneStackCalculateReverts(address owner, OrderConfig memory config) public {
        vm.assume(LibTestAddOrder.conformConfig(config, iDeployer));
        (bytes[] memory sources, uint256[] memory constants) =
            IParserV1(address(iDeployer)).parse("_:block-timestamp();:;");
        (constants);
        config.evaluableConfig.sources = sources;
        vm.expectRevert(abi.encodeWithSelector(MinFinalStack.selector, 2, 1));
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 2 for calculate order deploys.
    function testAddOrderRealTwoStackCalculateReverts(address owner, OrderConfig memory config) public {
        vm.assume(LibTestAddOrder.conformConfig(config, iDeployer));
        (bytes[] memory sources, uint256[] memory constants) =
            IParserV1(address(iDeployer)).parse("_ _:block-timestamp() chain-id();:;");
        (constants);
        config.evaluableConfig.sources = sources;
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }

    /// A stack of 3 for calculate order deploys.
    function testAddOrderRealThreeStackCalculate(address owner, OrderConfig memory config) public {
        vm.assume(LibTestAddOrder.conformConfig(config, iDeployer));
        (bytes[] memory sources, uint256[] memory constants) =
            IParserV1(address(iDeployer)).parse("_ _ _:block-timestamp() chain-id() block-number();:;");
        (constants);
        config.evaluableConfig.sources = sources;
        vm.prank(owner);
        iOrderbook.addOrder(config);
    }
}
