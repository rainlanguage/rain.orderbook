// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

contract OrderBookEvalTest is OrderBookExternalRealTest {
    function checkReentrancyRW() internal {
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // reads/writes for reentrancy guard.
        assert(reads.length == 3);
        reads[0] = bytes32(0);
        assert(writes.length == 2);
    }

    function testOrderBookEvalEmptyNoop() external {
        EvaluableV3[] memory evals = new EvaluableV3[](0);
        vm.record();
        iOrderbook.eval(evals);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == 0);
        assert(writes.length == 0);
    }

    function testOrderBookEvalOneStateless() external {
        EvaluableV3[] memory evals = new EvaluableV3[](1);
        evals[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2("_:1;"));
        vm.record();
        iOrderbook.eval(evals);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == 0);
        assert(writes.length == 0);
    }

    function testOrderBookEvalOneReadState() external {
        EvaluableV3[] memory evals = new EvaluableV3[](1);
        evals[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2("_:get(0);"));
        vm.record();
        iOrderbook.eval(evals);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        // 1 for get and 1 for set.
        assert(reads.length == 2);
        // 1 for the set implied by get.
        assert(writes.length == 1);
    }

    function testOrderBookEvalWriteStateSingle() external {
        EvaluableV3[] memory evals0 = new EvaluableV3[](1);
        evals0[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 2);"));
        vm.record();
        iOrderbook.eval(evals0);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        // 1 for the set.
        assert(reads.length == 1);
        assert(writes.length == 1);

        EvaluableV3[] memory evals1 = new EvaluableV3[](1);
        evals1[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 2) \"set works\");"));
        vm.record();
        iOrderbook.eval(evals1);
        checkReentrancyRW();
        (reads, writes) = vm.accesses(address(iStore));
        // 1 for the get and 1 for set.
        assert(reads.length == 2);
        // 1 for the set implied by get.
        assert(writes.length == 1);
    }

    function testOrderBookEvalWriteStateSequential() external {
        EvaluableV3[] memory evals0 = new EvaluableV3[](4);
        evals0[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 2);"));
        evals0[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 2) \"0th set not equal\");"));
        evals0[2] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(2 3);"));
        evals0[3] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 3) \"1st set not equal\");"));

        vm.record();
        iOrderbook.eval(evals0);

        checkReentrancyRW();

        // Again.
        EvaluableV3[] memory evals1 = new EvaluableV3[](4);
        evals1[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 20);"));
        evals1[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 20) \"0th set not equal\");"));
        evals1[2] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(2 30);"));
        evals1[3] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 30) \"1st set not equal\");"));
    }

    function testOrderBookEvalWriteStateDifferentOwnersNamespaced(address alice, address bob) external {
        vm.assume(alice != bob);

        EvaluableV3[] memory evals0 = new EvaluableV3[](4);
        evals0[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 2);"));
        evals0[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 2) \"0th set not equal\");"));
        evals0[2] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(2 3);"));
        evals0[3] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 3) \"1st set not equal\");"));

        vm.record();

        vm.prank(alice);
        iOrderbook.eval(evals0);

        checkReentrancyRW();

        EvaluableV3[] memory evals1 = new EvaluableV3[](4);
        evals1[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 20);"));
        evals1[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 20) \"0th set not equal\");"));
        evals1[2] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(2 30);"));
        evals1[3] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 30) \"1st set not equal\");"));

        vm.prank(bob);
        iOrderbook.eval(evals1);

        // Ensure that the state is different for different owners.
        EvaluableV3[] memory evals2 = new EvaluableV3[](2);
        evals2[0] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 2) \"alice state 1\");"));
        evals2[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 3) \"alice state 2\");"));

        vm.prank(alice);
        iOrderbook.eval(evals2);

        EvaluableV3[] memory evals3 = new EvaluableV3[](2);
        evals3[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 20) \"bob state 1\");"));
        evals3[1] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 30) \"bob state 2\");"));

        vm.prank(bob);
        iOrderbook.eval(evals3);
    }
}
