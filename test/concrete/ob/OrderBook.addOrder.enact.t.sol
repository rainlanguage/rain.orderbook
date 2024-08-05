// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3,
    EvaluableV3,
    ActionV1,
    OrderV3,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "src/lib/LibOrder.sol";

contract OrderBookAddOrderEnactTest is OrderBookExternalRealTest {
    using LibOrder for OrderV3;

    mapping(bytes32 => bool) nonces;

    function checkReentrancyRW(uint256 expectedReads, uint256 expectedWrites) internal {
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3 reads for reentrancy guard.
        // 1 reads for add order.
        assert(reads.length == expectedReads);
        assert(reads[0] == bytes32(uint256(0)));
        assert(reads[1] == bytes32(uint256(0)));
        assert(reads[reads.length - 1] == bytes32(uint256(0)));
        // 2 writes for reentrancy guard.
        // 1 write for add order.
        assert(writes.length == expectedWrites);
        assert(writes[0] == bytes32(uint256(0)));
        assert(writes[writes.length - 1] == bytes32(uint256(0)));
    }

    function checkAddOrder(
        address owner,
        OrderConfigV3 memory config,
        bytes[] memory evalStrings,
        uint256 expectedReads,
        uint256 expectedWrites
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.startPrank(owner);
        ActionV1[] memory actions = evalsToActions(evalStrings);
        vm.record();
        bool stateChanged = iOrderbook.addOrder2(config, actions);
        assert(stateChanged != nonces[config.nonce]);
        checkReentrancyRW(nonces[config.nonce] ? 4 : 5, nonces[config.nonce] ? 2 : 3);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
        nonces[config.nonce] = true;
    }

    function testAddOrderEmptyNoop(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals = new bytes[](0);
        checkAddOrder(alice, config, evals, 0, 0);
    }

    function testAddOrderOneStateless(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkAddOrder(alice, config, evals, 0, 0);
    }

    function testAddOrderOneReadState(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        checkAddOrder(alice, config, evals, 2, 1);
    }

    function testAddOrderWriteStateSingle(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals0 = new bytes[](1);
        evals0[0] = bytes(":set(1 2);");
        config.nonce = bytes32(uint256(0));
        checkAddOrder(alice, config, evals0, 1, 1);

        bytes[] memory evals1 = new bytes[](1);
        evals1[0] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        config.nonce = bytes32(uint256(1));
        checkAddOrder(alice, config, evals1, 2, 1);
    }

    function testAddOrderWriteStateSequential(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        checkAddOrder(alice, config, evals0, 6, 4);
    }

    function testAddOrderWriteStateDifferentOwnersNamespaced(address alice, address bob, OrderConfigV3 memory config)
        external
    {
        vm.assume(alice != bob);
        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        config.nonce = bytes32(uint256(0));
        checkAddOrder(alice, config, evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        config.nonce = bytes32(uint256(1));
        checkAddOrder(bob, config, evals1, 6, 4);

        bytes[] memory evals2 = new bytes[](2);
        evals2[0] = bytes(":ensure(equal-to(get(1) 2) \"alice state 1\");");
        evals2[1] = bytes(":ensure(equal-to(get(2) 3) \"alice state 2\");");
        config.nonce = bytes32(uint256(2));
        checkAddOrder(alice, config, evals2, 4, 2);

        bytes[] memory evals3 = new bytes[](2);
        evals3[0] = bytes(":ensure(equal-to(get(1) 20) \"bob state 1\");");
        evals3[1] = bytes(":ensure(equal-to(get(2) 30) \"bob state 2\");");
        config.nonce = bytes32(uint256(3));
        checkAddOrder(bob, config, evals3, 4, 2);
    }

    /// Evals DO NOT eval if the adding of an order is a noop.
    /// I.e. if the order is added twice the second time nothing happens.
    function testAddLiveOrderNoop(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals0 = new bytes[](0);
        checkAddOrder(alice, config, evals0, 0, 0);

        // The config is the same here so the same order is added.
        // This is a noop so the evals do not run.
        bytes[] memory evals1 = new bytes[](1);
        evals1[0] = bytes(":ensure(0 \"always error\");");
        checkAddOrder(alice, config, evals1, 0, 0);
    }

    /// A revert in the action prevents the order being added.
    /// forge-config: default.assertions_revert = false
    /// forge-config: default.legacy_assertions = true
    function testAddLiveOrderRevertNoAdd(address alice, OrderConfigV3 memory config) external {
        bytes[] memory evals0 = new bytes[](1);
        evals0[0] = bytes(":ensure(0 \"always revert\");");

        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.startPrank(alice);
        ActionV1[] memory actions = evalsToActions(evals0);
        vm.expectRevert("always revert");
        bool stateChanged = iOrderbook.addOrder2(config, actions);
        assert(!stateChanged);

        OrderV3 memory order = OrderV3(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        assert(!iOrderbook.orderExists(order.hash()));
    }
}
