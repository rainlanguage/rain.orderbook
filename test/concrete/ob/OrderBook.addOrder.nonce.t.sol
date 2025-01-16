// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, OrderV3, EvaluableV3, TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "src/lib/LibOrder.sol";

contract OrderBookAddOrderNonceTest is OrderBookExternalRealTest {
    using LibOrder for OrderV3;

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderNonceSameOrderNoop(address owner, OrderConfigV3 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        OrderV3 memory order = OrderV3(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(owner);
        bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
        assert(stateChange);
        assert(iOrderbook.orderExists(order.hash()));

        vm.prank(owner);
        stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
        assert(!stateChange);
        assert(iOrderbook.orderExists(order.hash()));
    }

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderNonceDifferentNonceStateChange(address owner, OrderConfigV3 memory config, bytes32 otherNonce)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(config.nonce != otherNonce);
        vm.prank(owner);
        bool stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
        assert(stateChange);
        assert(
            iOrderbook.orderExists(
                OrderV3(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce).hash()
            )
        );

        config.nonce = otherNonce;
        vm.prank(owner);
        stateChange = iOrderbook.addOrder2(config, new TaskV1[](0));
        assert(stateChange);
        assert(
            iOrderbook.orderExists(
                OrderV3(owner, config.evaluable, config.validInputs, config.validOutputs, otherNonce).hash()
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderNonceSameNonceDifferentOrderStateChange(
        address owner,
        OrderConfigV3 memory config0,
        OrderConfigV3 memory config1
    ) public {
        LibTestAddOrder.conformConfig(config0, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(config1, iInterpreter, iStore);

        config1.nonce = config0.nonce;
        vm.assume(keccak256(abi.encode(config0)) != keccak256(abi.encode(config1)));
        vm.prank(owner);
        bool stateChange = iOrderbook.addOrder2(config0, new TaskV1[](0));
        assert(stateChange);
        assert(
            iOrderbook.orderExists(
                OrderV3(owner, config0.evaluable, config0.validInputs, config0.validOutputs, config0.nonce).hash()
            )
        );

        vm.prank(owner);
        stateChange = iOrderbook.addOrder2(config1, new TaskV1[](0));
        assert(stateChange);
        assert(
            iOrderbook.orderExists(
                OrderV3(owner, config1.evaluable, config1.validInputs, config1.validOutputs, config1.nonce).hash()
            )
        );
    }
}
