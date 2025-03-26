// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV4, OrderV4, EvaluableV4, TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";

contract OrderBookRemoveOrderOwnerTest is OrderBookExternalRealTest {
    using LibOrder for OrderV4;

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOwnerSameOrderNoop(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        OrderV4 memory order = OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.startPrank(owner);

        for (uint256 i = 0; i < 2; i++) {
            bool stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(order.hash()));
            stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));
            assert(!stateChange);
            assert(iOrderbook.orderExists(order.hash()));

            stateChange = iOrderbook.removeOrder2(order, new TaskV2[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(order.hash()));
            stateChange = iOrderbook.removeOrder2(order, new TaskV2[](0));
            assert(!stateChange);
            assert(!iOrderbook.orderExists(order.hash()));
        }

        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOwnerDifferentOwnerStateChange(OrderConfigV4 memory config, address alice, address bob)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV4 memory orderAlice =
            OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        OrderV4 memory orderBob = OrderV4(bob, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        {
            vm.prank(alice);
            bool stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));

            assert(stateChange);
            assert(iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new TaskV2[](0));
            assert(!stateChange);
            assert(iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));

            vm.prank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new TaskV2[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));
        }

        {
            vm.prank(bob);
            bool stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));

            vm.prank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new TaskV2[](0));
            assert(!stateChange);
            assert(iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new TaskV2[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));
        }

        {
            vm.prank(alice);
            bool stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(orderBob.hash()));
            assert(iOrderbook.orderExists(orderAlice.hash()));

            vm.prank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new TaskV2[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderAlice.hash()));
            assert(iOrderbook.orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new TaskV2[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));
        }
    }

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderWrongOwner(OrderConfigV4 memory config, address alice, address bob) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV4 memory order = OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(alice);
        bool stateChange = iOrderbook.addOrder3(config, new TaskV2[](0));
        assert(stateChange);
        assert(iOrderbook.orderExists(order.hash()));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        stateChange = iOrderbook.removeOrder2(order, new TaskV2[](0));
        assert(!stateChange);
        assert(iOrderbook.orderExists(order.hash()));
    }
}
