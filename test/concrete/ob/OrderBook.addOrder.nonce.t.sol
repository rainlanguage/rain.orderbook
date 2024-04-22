// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";

contract OrderBookAddOrderNonceTest is OrderBookExternalRealTest {
    function testAddOrderNonceSameOrderNoop(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.startPrank(owner);
        bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(!stateChange);
        vm.stopPrank();
    }

    function testAddOrderNonceDifferentNonceStateChange(address owner, OrderConfigV3 memory config, bytes32 otherNonce)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(config.nonce != otherNonce);
        vm.startPrank(owner);
        bool stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        config.nonce = otherNonce;
        stateChange = iOrderbook.addOrder2(config, new EvaluableV3[](0));
        assert(stateChange);
        vm.stopPrank();
    }

    function testAddOrderNonceSameNonceDifferentOrderStateChange(
        address owner,
        OrderConfigV3 memory config0,
        OrderConfigV3 memory config1
    ) public {
        LibTestAddOrder.conformConfig(config0, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(config1, iInterpreter, iStore);
        config1.nonce = config0.nonce;
        vm.assume(keccak256(abi.encode(config0)) != keccak256(abi.encode(config1)));
        vm.startPrank(owner);
        bool stateChange = iOrderbook.addOrder2(config0, new EvaluableV3[](0));
        assert(stateChange);
        stateChange = iOrderbook.addOrder2(config1, new EvaluableV3[](0));
        assert(stateChange);
        vm.stopPrank();
    }
}
