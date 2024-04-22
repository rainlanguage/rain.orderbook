// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, OrderV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";

contract OrderBookRemoveOrderOwnerTest is OrderBookExternalRealTest {
    using LibOrder for OrderV3;

    function testRemoveOrderOwnerSameOrderNoop(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        OrderV3 memory order = OrderV3(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.startPrank(owner);

        for (uint256 i = 0; i < 2; i++) {
            bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
            assert(stateChange);
            stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
            assert(!stateChange);

            stateChange = iOrderbook.removeOrder2(order, new EvaluableV3[](0));
            assert(stateChange);
            stateChange = iOrderbook.removeOrder2(order, new EvaluableV3[](0));
            assert(!stateChange);
        }

        vm.stopPrank();
    }

    function testRemoveOrderOwnerDifferentOwnerStateChange(OrderConfigV3 memory config, address alice, address bob)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV3 memory orderAlice =
            OrderV3(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        OrderV3 memory orderBob = OrderV3(bob, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        {
            vm.startPrank(alice);
            bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(iOrderbook.orderExists(orderAlice.hash()));

            vm.startPrank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new EvaluableV3[](0));
            assert(!stateChange);
            vm.stopPrank();

            assert(iOrderbook.orderExists(orderAlice.hash()));

            vm.startPrank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(!iOrderbook.orderExists(orderAlice.hash()));
        }

        {
            vm.startPrank(bob);
            bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(iOrderbook.orderExists(orderBob.hash()));

            vm.startPrank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new EvaluableV3[](0));
            assert(!stateChange);
            vm.stopPrank();

            assert(iOrderbook.orderExists(orderBob.hash()));

            vm.startPrank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(!iOrderbook.orderExists(orderBob.hash()));
        }

        {
            vm.startPrank(alice);
            bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(iOrderbook.orderExists(orderAlice.hash()));

            vm.startPrank(bob);
            stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(iOrderbook.orderExists(orderBob.hash()));

            vm.startPrank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();

            assert(!iOrderbook.orderExists(orderAlice.hash()));

            vm.startPrank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new EvaluableV3[](0));
            assert(stateChange);
            vm.stopPrank();
        }
    }

    function testRemoveOrderWrongOwner(OrderConfigV3 memory config, address alice, address bob) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV3 memory order = OrderV3(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.startPrank(alice);
        bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        vm.stopPrank();

        assert(iOrderbook.orderExists(order.hash()));

        vm.startPrank(bob);
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        stateChange = iOrderbook.removeOrder2(order, new EvaluableV3[](0));
        assert(!stateChange);
        vm.stopPrank();
    }
}
