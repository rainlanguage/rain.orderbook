// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3,
    EvaluableV3,
    ActionV1,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";

contract OrderBookEnactTest is OrderBookExternalRealTest {
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

    function checkEnact(address owner, bytes[] memory evalStrings, uint256 expectedReads, uint256 expectedWrites)
        internal
    {
        vm.startPrank(owner);
        ActionV1[] memory actions = new ActionV1[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                ActionV1(EvaluableV3(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        vm.record();
        iOrderbook.enact(actions);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
    }

    function testOrderBookEvalEmptyNoop(address alice) external {
        checkEnact(alice, new bytes[](0), 0, 0);
    }

    function testOrderBookEvalOneStateless(address alice) external {
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkEnact(alice, evals, 0, 0);
    }

    function testOrderBookEvalOneReadState(address alice) external {
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        checkEnact(alice, evals, 2, 1);
    }

    function testOrderBookEvalWriteStateSingle(address alice) external {
        bytes[] memory evals0 = new bytes[](1);
        evals0[0] = bytes(":set(1 2);");
        checkEnact(alice, evals0, 1, 1);

        bytes[] memory evals1 = new bytes[](1);
        evals1[0] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        checkEnact(alice, evals1, 2, 1);
    }

    function testOrderBookEvalWriteStateSequential() external {
        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        checkEnact(address(0), evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        checkEnact(address(0), evals1, 6, 4);
    }

    function testOrderBookEvalWriteStateDifferentOwnersNamespaced(address alice, address bob) external {
        vm.assume(alice != bob);
        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        checkEnact(alice, evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        checkEnact(bob, evals1, 6, 4);

        bytes[] memory evals2 = new bytes[](2);
        evals2[0] = bytes(":ensure(equal-to(get(1) 2) \"alice state 1\");");
        evals2[1] = bytes(":ensure(equal-to(get(2) 3) \"alice state 2\");");
        checkEnact(alice, evals2, 4, 2);

        bytes[] memory evals3 = new bytes[](2);
        evals3[0] = bytes(":ensure(equal-to(get(1) 20) \"bob state 1\");");
        evals3[1] = bytes(":ensure(equal-to(get(2) 30) \"bob state 2\");");
        checkEnact(bob, evals3, 4, 2);
    }
}
