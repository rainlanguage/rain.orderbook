// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3, EvaluableV3, ActionV1, SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";

contract OrderBookDepositEnactTest is OrderBookExternalRealTest {
    function checkReentrancyRW() internal {
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3 reads for reentrancy guard.
        // 2 reads for deposit.
        assert(reads.length == 5);
        assert(reads[0] == bytes32(uint256(0)));
        assert(reads[1] == bytes32(uint256(0)));
        assert(reads[4] == bytes32(uint256(0)));
        // 2 writes for reentrancy guard.
        // 1 write for deposit.
        assert(writes.length == 3);
        assert(writes[0] == bytes32(uint256(0)));
        assert(writes[2] == bytes32(uint256(0)));
    }

    function checkDeposit(
        address owner,
        uint256 vaultId,
        uint256 amount,
        bytes[] memory evalStrings,
        uint256 expectedReads,
        uint256 expectedWrites
    ) internal {
        vm.startPrank(owner);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iOrderbook), amount),
            abi.encode(true)
        );

        ActionV1[] memory actions = new ActionV1[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                ActionV1(EvaluableV3(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        vm.record();
        iOrderbook.deposit2(address(iToken0), vaultId, amount, actions);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
    }

    function testOrderBookDepositEnactEmptyNoop(address alice, uint256 vaultId, uint256 amount) external {
        vm.assume(amount > 0);
        bytes[] memory evals = new bytes[](0);
        checkDeposit(alice, vaultId, amount, evals, 0, 0);
    }

    function testOrderBookDepositEnactOneStateless(address alice, uint256 vaultId, uint256 amount) external {
        vm.assume(amount > 0);
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkDeposit(alice, vaultId, amount, evals, 0, 0);
    }

    function testOrderBookDepositEnactOneReadState(address alice, uint256 vaultId, uint256 amount) external {
        vm.assume(amount > 0);
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each get is 1 write.
        checkDeposit(alice, vaultId, amount, evals, 2, 1);
    }

    function testOrderBookDepositEvalWriteStateSingle(address alice, uint256 vaultId, uint256 amount) external {
        amount = bound(amount, 1, type(uint128).max);
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":set(1 2);");
        // 1 for the set.
        checkDeposit(alice, vaultId, amount, evals, 1, 1);

        evals[0] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        checkDeposit(alice, vaultId, amount, evals, 2, 1);
    }

    function testOrderBookDepositEvalWriteStateSequential(address alice, uint256 vaultId, uint256 amount) external {
        amount = bound(amount, 1, type(uint128).max);
        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        checkDeposit(alice, vaultId, amount, evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        checkDeposit(alice, vaultId, amount, evals1, 6, 4);
    }

    function testOrderBookDepositEvalWriteStateDifferentOwnersNamespaced(
        address alice,
        address bob,
        uint256 vaultId,
        uint256 amount
    ) external {
        vm.assume(alice != bob);
        amount = bound(amount, 1, type(uint128).max);

        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        checkDeposit(alice, vaultId, amount, evals0, 6, 4);

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        checkDeposit(bob, vaultId, amount, evals1, 6, 4);

        bytes[] memory evals2 = new bytes[](2);
        evals2[0] = bytes(":ensure(equal-to(get(1) 2) \"alice state 1\");");
        evals2[1] = bytes(":ensure(equal-to(get(2) 3) \"alice state 2\");");
        checkDeposit(alice, vaultId, amount, evals2, 4, 2);

        bytes[] memory evals3 = new bytes[](2);
        evals3[0] = bytes(":ensure(equal-to(get(1) 20) \"bob state 1\");");
        evals3[1] = bytes(":ensure(equal-to(get(2) 30) \"bob state 2\");");
        checkDeposit(bob, vaultId, amount, evals3, 4, 2);
    }

    /// A revert in the action prevents the deposit from being enacted.
    function testDepositRevertInAction(address alice, uint256 vaultId, uint256 amount) external {
        vm.assume(amount != 0);
        vm.startPrank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), amount),
            abi.encode(true)
        );

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"revert in action\");");

        ActionV1[] memory actions = evalsToActions(evals);

        assertEq(0, iOrderbook.vaultBalance(alice, address(iToken0), vaultId));

        vm.expectRevert("revert in action");
        iOrderbook.deposit2(address(iToken0), vaultId, amount, actions);

        assertEq(0, iOrderbook.vaultBalance(alice, address(iToken0), vaultId));
    }
}
