// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

contract OrderBookEvalTest is OrderBookExternalRealTest {
    function checkReentrancyRW() internal {
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // reads/writes for reentrancy guard.
        assert(reads.length == 3);
        assert(reads[0] == bytes32(uint256(0)));
        assert(reads[1] == bytes32(uint256(0)));
        assert(reads[2] == bytes32(uint256(0)));
        assert(writes.length == 2);
        assert(writes[0] == bytes32(uint256(0)));
        assert(writes[1] == bytes32(uint256(0)));
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
        // each get is 2 reads. 1 during eval and 1 during store set.
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
        // each get is 2 reads. 1 during eval and 1 during store set.
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

        (bytes32[] memory reads0, bytes32[] memory writes0) = vm.accesses(address(iStore));
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each set is 1 read.
        assert(reads0.length == 6);
        // each get is 1 write.
        // each set is 1 write.
        assert(writes0.length == 4);

        // Again.
        EvaluableV3[] memory evals1 = new EvaluableV3[](4);
        evals1[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 20);"));
        evals1[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 20) \"0th set not equal\");"));
        evals1[2] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(2 30);"));
        evals1[3] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 30) \"1st set not equal\");"));

        vm.record();
        iOrderbook.eval(evals1);

        checkReentrancyRW();

        (bytes32[] memory reads1, bytes32[] memory writes1) = vm.accesses(address(iStore));
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each set is 1 read.
        assert(reads1.length == 6);
        // each get is 1 write.
        // each set is 1 write.
        assert(writes1.length == 4);
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

        (bytes32[] memory reads0, bytes32[] memory writes0) = vm.accesses(address(iStore));
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each set is 1 read.
        assert(reads0.length == 6);
        // each get is 1 write.
        // each set is 1 write.
        assert(writes0.length == 4);

        EvaluableV3[] memory evals1 = new EvaluableV3[](4);
        evals1[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(1 20);"));
        evals1[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 20) \"0th set not equal\");"));
        evals1[2] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":set(2 30);"));
        evals1[3] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 30) \"1st set not equal\");"));

        vm.record();
        vm.prank(bob);
        iOrderbook.eval(evals1);

        checkReentrancyRW();

        (bytes32[] memory reads1, bytes32[] memory writes1) = vm.accesses(address(iStore));
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each set is 1 read.
        assert(reads1.length == 6);
        // each get is 1 write.
        // each set is 1 write.
        assert(writes1.length == 4);

        // Ensure that the state is different for different owners.
        EvaluableV3[] memory evals2 = new EvaluableV3[](2);
        evals2[0] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 2) \"alice state 1\");"));
        evals2[1] =
            EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 3) \"alice state 2\");"));

        vm.record();
        vm.prank(alice);
        iOrderbook.eval(evals2);

        checkReentrancyRW();

        (bytes32[] memory reads2, bytes32[] memory writes2) = vm.accesses(address(iStore));
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each set is 1 read.
        assert(reads2.length == 4);
        // each get is 1 write.
        // each set is 1 write.
        assert(writes2.length == 2);

        EvaluableV3[] memory evals3 = new EvaluableV3[](2);
        evals3[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(1) 20) \"bob state 1\");"));
        evals3[1] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(":ensure(equal-to(get(2) 30) \"bob state 2\");"));

        vm.record();
        vm.prank(bob);
        iOrderbook.eval(evals3);

        (bytes32[] memory reads3, bytes32[] memory writes3) = vm.accesses(address(iStore));
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each set is 1 read.
        assert(reads3.length == 4);
        // each get is 1 write.
        // each set is 1 write.
        assert(writes3.length == 2);
    }
}
