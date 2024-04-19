// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

contract OrderBookEvalTest is OrderBookExternalRealTest {
    function testOrderBookEvalEmptyNoop() external {
        EvaluableV3[] memory evals = new EvaluableV3[](0);
        vm.record();
        iOrderbook.eval(evals);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // reads/writes for reentrancy guard.
        assert(reads.length == 3);
        assert(writes.length == 2);
        (reads, writes) = vm.accesses(address(iStore));
        assert(reads.length == 0);
        assert(writes.length == 0);
    }

    function testOrderBookEvalOneStateless() external {
        EvaluableV3[] memory evals = new EvaluableV3[](1);
        evals[0] = EvaluableV3(
            iInterpreter,
            iStore,
            iParserV2.parse2("_:1;")
        );
        vm.record();
        iOrderbook.eval(evals);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // reads/writes for reentrancy guard.
        assert(reads.length == 3);
        assert(writes.length == 2);
        (reads, writes) = vm.accesses(address(iStore));
        assert(reads.length == 0);
        assert(writes.length == 0);
    }

    function testOrderBookEvalOneReadState() external {
        EvaluableV3[] memory evals = new EvaluableV3[](1);
        evals[0] = EvaluableV3(
            iInterpreter,
            iStore,
            iParserV2.parse2("_:get(0);")
        );
        vm.record();
        iOrderbook.eval(evals);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // reads/writes for reentrancy guard.
        assert(reads.length == 3);
        assert(writes.length == 2);
        (reads, writes) = vm.accesses(address(iStore));
        // 1 for get and 1 for set.
        assert(reads.length == 2);
        // 1 for the set implied by get.
        assert(writes.length == 1);
    }
}