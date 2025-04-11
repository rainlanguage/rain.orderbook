// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV4,
    EvaluableV4,
    TaskV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract OrderBookWithdrawEvalTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;
    using LibDecimalFloat for Float;

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
        bytes32 vaultId,
        Float memory depositAmount,
        Float memory targetAmount,
        bytes[] memory evalStrings,
        uint256 expectedReads,
        uint256 expectedWrites
    ) internal {
        uint256 depositAmount18 = depositAmount.toFixedDecimalLossless(18);
        uint256 targetAmount18 = targetAmount.toFixedDecimalLossless(18);

        vm.startPrank(owner);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iOrderbook), depositAmount18),
            abi.encode(true)
        );

        if (depositAmount18 > 0) {
            iOrderbook.deposit3(address(iToken0), vaultId, depositAmount, new TaskV2[](0));
        }

        TaskV2[] memory actions = new TaskV2[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                TaskV2(EvaluableV4(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        uint256 withdrawAmount18 = depositAmount18 > targetAmount18 ? targetAmount18 : depositAmount18;
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, owner, withdrawAmount18),
            abi.encode(true)
        );
        vm.record();
        iOrderbook.withdraw3(address(iToken0), vaultId, targetAmount, actions);
        checkReentrancyRW(depositAmount18 > 0 ? 5 : 4, depositAmount18 > 0 ? 3 : 2);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalEmptyNoop(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);
        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, new bytes[](0), 0, 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalOneStateless(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals, 0, 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalOneReadState(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals, 2, 1);
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalWriteStateSingle(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

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
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

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
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(alice != bob);
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

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
    function testOrderBookWithdrawalEvalZeroAmountEvalNoop(address alice, bytes32 vaultId, uint256 withdrawAmount18)
        external
    {
        withdrawAmount18 = bound(withdrawAmount18, 1, type(uint128).max);

        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"always fails\");");
        checkWithdraw(alice, vaultId, Float(0, 0), withdrawAmount, evals, 0, 0);
    }

    /// A revert in the action prevents withdraw from being enacted.
    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawalEvalRevertInAction(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(withdrawAmount18, 18);

        vm.startPrank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit3(address(iToken0), vaultId, depositAmount, new TaskV2[](0));

        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount), abi.encode(true)
        );

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"revert in action\");");
        TaskV2[] memory actions = evalsToActions(evals);

        assertTrue(depositAmount.eq(iOrderbook.vaultBalance2(alice, address(iToken0), vaultId)));

        vm.expectRevert("revert in action");
        iOrderbook.withdraw3(address(iToken0), vaultId, withdrawAmount, actions);

        assertTrue(depositAmount.eq(iOrderbook.vaultBalance2(alice, address(iToken0), vaultId)));
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderWithdrawContext(address alice, bytes32 vaultId, uint256 depositAmount18, uint256 targetAmount18)
        external
    {
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        targetAmount18 = bound(targetAmount18, 1, type(uint128).max);
        uint256 withdrawAmount18 = depositAmount18 > targetAmount18 ? targetAmount18 : depositAmount18;

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        Float memory targetAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(targetAmount18, 18);

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
                uint256(vaultId).toHexString(),
                ") \"withdraw vaultId\");"
            )
        );
        evals[4] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-vault-balance() ",
                depositAmount18.toString(),
                "e-6) \"vault balance\");"
            )
        );
        evals[5] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-amount() ",
                withdrawAmount18.toString(),
                "e-6) \"withdraw amount\");"
            )
        );
        // target amount
        evals[6] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-target-amount() ",
                targetAmount18.toString(),
                "e-6) \"target amount\");"
            )
        );
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));

        checkWithdraw(alice, vaultId, depositAmount, targetAmount, evals, 0, 0);
    }
}
