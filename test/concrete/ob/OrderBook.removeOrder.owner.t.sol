// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, OrderV3, EvaluableV3, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";

contract OrderBookRemoveOrderOwnerTest is OrderBookExternalRealTest {
    using LibOrder for OrderV3;

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOwnerSameOrderNoop(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        OrderV3 memory order = OrderV3(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.startPrank(owner);

        for (uint256 i = 0; i < 2; i++) {
            bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(order.hash()));
            stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
            assert(!stateChange);
            assert(iOrderbook.orderExists(order.hash()));

            stateChange = iOrderbook.removeOrder2(order, new TaskV1[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(order.hash()));
            stateChange = iOrderbook.removeOrder2(order, new TaskV1[](0));
            assert(!stateChange);
            assert(!iOrderbook.orderExists(order.hash()));
        }

        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOwnerDifferentOwnerStateChange(OrderConfigV3 memory config, address alice, address bob)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV3 memory orderAlice =
            OrderV3(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        OrderV3 memory orderBob = OrderV3(bob, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        {
            vm.prank(alice);
            bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));

            assert(stateChange);
            assert(iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new TaskV1[](0));
            assert(!stateChange);
            assert(iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));

            vm.prank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new TaskV1[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));
        }

        {
            vm.prank(bob);
            bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));

            vm.prank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new TaskV1[](0));
            assert(!stateChange);
            assert(iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new TaskV1[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));
        }

        {
            vm.prank(alice);
            bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(orderAlice.hash()));
            assert(!iOrderbook.orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
            assert(stateChange);
            assert(iOrderbook.orderExists(orderBob.hash()));
            assert(iOrderbook.orderExists(orderAlice.hash()));

            vm.prank(alice);
            stateChange = iOrderbook.removeOrder2(orderAlice, new TaskV1[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderAlice.hash()));
            assert(iOrderbook.orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = iOrderbook.removeOrder2(orderBob, new TaskV1[](0));
            assert(stateChange);
            assert(!iOrderbook.orderExists(orderBob.hash()));
            assert(!iOrderbook.orderExists(orderAlice.hash()));
        }
    }

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderWrongOwner(OrderConfigV3 memory config, address alice, address bob) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV3 memory order = OrderV3(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(alice);
        bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
        assert(stateChange);
        assert(iOrderbook.orderExists(order.hash()));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, bob, alice));
        stateChange = iOrderbook.removeOrder2(order, new TaskV1[](0));
        assert(!stateChange);
        assert(iOrderbook.orderExists(order.hash()));
    }
}
