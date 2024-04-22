// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";

contract OrderBookAddOrderOwnerTest is OrderBookExternalRealTest {
    function testAddOrderOwnerSameOrderNoop(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        vm.startPrank(owner);
        bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(!stateChange);
        vm.stopPrank();
    }

    function testAddOrderOwnerDifferentOwnerStateChange(OrderConfigV3 memory config, address alice, address bob)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        vm.startPrank(alice);
        bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        vm.stopPrank();

        vm.startPrank(bob);
        stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        vm.stopPrank();
    }
}
