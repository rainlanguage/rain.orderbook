// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV3, EvaluableV3, TaskV1, SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

contract OrderBookWithdrawEvalTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;

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
        uint256 targetAmount,
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
            iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new TaskV1[](0));
        }

        TaskV1[] memory actions = new TaskV1[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                TaskV1(EvaluableV3(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        uint256 withdrawAmount = depositAmount > targetAmount ? targetAmount : depositAmount;
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, owner, withdrawAmount), abi.encode(true)
        );
        vm.record();
        iOrderbook.withdraw2(address(iToken0), vaultId, targetAmount, actions);
        checkReentrancyRW(depositAmount > 0 ? 5 : 4, depositAmount > 0 ? 3 : 2);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 100
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

    /// forge-config: default.fuzz.runs = 100
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

    /// forge-config: default.fuzz.runs = 100
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

    /// forge-config: default.fuzz.runs = 100
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

    /// forge-config: default.fuzz.runs = 100
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

    /// forge-config: default.fuzz.runs = 100
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
    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawalEvalZeroAmountEvalNoop(address alice, uint256 vaultId, uint256 withdrawAmount)
        external
    {
        withdrawAmount = bound(withdrawAmount, 1, type(uint128).max);
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"always fails\");");
        checkWithdraw(alice, vaultId, 0, withdrawAmount, evals, 0, 0);
    }

    /// A revert in the action prevents withdraw from being enacted.
    /// forge-config: default.fuzz.runs = 100
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
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new TaskV1[](0));

        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount), abi.encode(true)
        );

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"revert in action\");");
        TaskV1[] memory actions = evalsToActions(evals);

        assertEq(depositAmount, iOrderbook.vaultBalance(alice, address(iToken0), vaultId));

        vm.expectRevert("revert in action");
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, actions);

        assertEq(depositAmount, iOrderbook.vaultBalance(alice, address(iToken0), vaultId));
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderWithdrawContext(address alice, uint256 vaultId, uint256 depositAmount, uint256 targetAmount)
        external
    {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        targetAmount = bound(targetAmount, 1, type(uint128).max);
        uint256 withdrawAmount = depositAmount > targetAmount ? targetAmount : depositAmount;

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes[] memory evals = new bytes[](7);
        evals[0] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(orderbook() ",
                address(iOrderbook).toHexString(),
                ") \"orderbook is iOrderbook\");"
            )
        );
        evals[1] = bytes(
            string.concat(
                usingWordsFrom, ":ensure(equal-to(withdrawer() ", alice.toHexString(), ") \"withdrawer is alice\");"
            )
        );
        evals[2] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-token() ",
                address(iToken0).toHexString(),
                ") \"withdraw token\");"
            )
        );
        evals[3] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-vault-id() ",
                vaultId.toHexString(),
                ") \"withdraw vaultId\");"
            )
        );
        evals[4] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-vault-balance() ",
                depositAmount.toString(),
                "e-6) \"vault balance\");"
            )
        );
        evals[5] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-amount() ",
                withdrawAmount.toString(),
                "e-6) \"withdraw amount\");"
            )
        );
        // target amount
        evals[6] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-target-amount() ",
                targetAmount.toString(),
                "e-6) \"target amount\");"
            )
        );
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));

        checkWithdraw(alice, vaultId, depositAmount, targetAmount, evals, 0, 0);
    }
}
