// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3,
    EvaluableV3,
    ActionV1,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";

contract OrderBookWithdrawEvalTest is OrderBookExternalRealTest {
    function checkReentrancyRW(uint256 expectedReads, uint256 expectedWrites) internal {
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3 reads for reentrancy guard.
        // 2 reads for deposit.
        assert(reads.length == expectedReads);
        assert(reads[0] == bytes32(uint256(0)));
        assert(reads[1] == bytes32(uint256(0)));
        assert(reads[reads.length - 1] == bytes32(uint256(0)));
        // 2 writes for reentrancy guard.
        // 1 write for deposit.
        assert(writes.length == expectedWrites);
        assert(writes[0] == bytes32(uint256(0)));
        assert(writes[writes.length - 1] == bytes32(uint256(0)));
    }

    function checkWithdraw(
        address owner,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount,
        bytes[] memory evalStrings,
        uint256 expectedReads,
        uint256 expectedWrites
    ) internal {
        vm.startPrank(owner);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        if (depositAmount > 0) {
            iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new ActionV1[](0));
        }

        ActionV1[] memory actions = new ActionV1[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                ActionV1(EvaluableV3(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, owner, withdrawAmount), abi.encode(true)
        );
        vm.record();
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, actions);
        checkReentrancyRW(depositAmount > 0 ? 5 : 4, depositAmount > 0 ? 3 : 2);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
    }

    function testOrderBookWithdrawEvalEmptyNoop(
        address alice,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, new bytes[](0), 0, 0);
    }

    function testOrderBookWithdrawEvalOneStateless(
        address alice,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals, 0, 0);
    }

    function testOrderBookWithdrawEvalOneReadState(
        address alice,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals, 2, 1);
    }

    function testOrderBookWithdrawEvalWriteStateSingle(
        address alice,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        bytes[] memory evals0 = new bytes[](1);
        evals0[0] = bytes(":set(1 2);");
        // each set is 1 read and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals0, 1, 1);

        bytes[] memory evals1 = new bytes[](1);
        evals1[0] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals1, 2, 1);
    }

    function testOrderBookWithdrawEvalWriteStateSequential(
        address alice,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals1, 6, 4);
    }

    function testOrderBookWithdrawEvalWriteStateDifferentOwnersNamespaced(
        address alice,
        address bob,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        vm.assume(alice != bob);
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(bob, vaultId, depositAmount, withdrawAmount, evals1, 6, 4);

        bytes[] memory evals2 = new bytes[](2);
        evals2[0] = bytes(":ensure(equal-to(get(1) 2) \"alice state 1\");");
        evals2[1] = bytes(":ensure(equal-to(get(2) 3) \"alice state 2\");");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals2, 4, 2);

        bytes[] memory evals3 = new bytes[](2);
        evals3[0] = bytes(":ensure(equal-to(get(1) 20) \"bob state 1\");");
        evals3[1] = bytes(":ensure(equal-to(get(2) 30) \"bob state 2\");");
        // each get is 2 reads and 1 write.
        checkWithdraw(bob, vaultId, depositAmount, withdrawAmount, evals3, 4, 2);
    }

    /// Evals DO NOT run if withdrawal amount ends up as 0.
    /// No withdraw => no eval.
    function testOrderBookWithdrawalEvalZeroAmountEvalNoop(address alice, uint256 vaultId, uint256 withdrawAmount)
        external
    {
        withdrawAmount = bound(withdrawAmount, 1, type(uint128).max);
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"always fails\");");
        checkWithdraw(alice, vaultId, 0, withdrawAmount, evals, 0, 0);
    }

    /// A revert in the action prevents withdraw from being enacted.
    function testOrderBookWithdrawalEvalRevertInAction(
        address alice,
        uint256 vaultId,
        uint256 depositAmount,
        uint256 withdrawAmount
    ) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        vm.startPrank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new ActionV1[](0));

        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount), abi.encode(true)
        );

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"revert in action\");");
        ActionV1[] memory actions = evalsToActions(evals);

        assertEq(depositAmount, iOrderbook.vaultBalance(alice, address(iToken0), vaultId));

        vm.expectRevert("revert in action");
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, actions);

        assertEq(depositAmount, iOrderbook.vaultBalance(alice, address(iToken0), vaultId));
    }
}
