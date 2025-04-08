// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest, LibDecimalFloat, Float} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderConfigV4,
    EvaluableV4,
    TaskV2,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {LibFormatDecimalFloat} from "rain.math.float/lib/format/LibFormatDecimalFloat.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

contract OrderBookDepositEnactTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;
    using LibDecimalFloat for Float;
    using LibFormatDecimalFloat for Float;

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
        bytes32 vaultId,
        Float memory amount,
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

        TaskV2[] memory actions = new TaskV2[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                TaskV2(EvaluableV4(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        vm.record();
        iOrderbook.deposit3(address(iToken0), vaultId, amount, actions);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 10
    function testOrderBookDepositEnactEmptyNoop(address alice, bytes32 vaultId, Float memory amount) external {
        vm.assume(amount.gt(Float(0, 0)));
        bytes[] memory evals = new bytes[](0);
        checkDeposit(alice, vaultId, amount, evals, 0, 0);
    }

    /// forge-config: default.fuzz.runs = 10
    function testOrderBookDepositEnactOneStateless(address alice, bytes32 vaultId, Float memory amount) external {
        vm.assume(amount.gt(Float(0, 0)));
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkDeposit(alice, vaultId, amount, evals, 0, 0);
    }

    /// forge-config: default.fuzz.runs = 10
    function testOrderBookDepositEnactOneReadState(address alice, bytes32 vaultId, Float memory amount) external {
        vm.assume(amount.gt(Float(0, 0)));
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        // each get is 2 reads. 1 during eval and 1 during store set.
        // each get is 1 write.
        checkDeposit(alice, vaultId, amount, evals, 2, 1);
    }

    /// forge-config: default.fuzz.runs = 10
    function testOrderBookDepositEvalWriteStateSingle(address alice, bytes32 vaultId, int256 amount18) external {
        amount18 = bound(amount18, 1, type(int128).max);
        Float memory amount = Float(amount18, -18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":set(1 2);");
        // 1 for the set.
        checkDeposit(alice, vaultId, amount, evals, 1, 1);

        evals[0] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        checkDeposit(alice, vaultId, amount, evals, 2, 1);
    }

    /// forge-config: default.fuzz.runs = 10
    function testOrderBookDepositEvalWriteStateSequential(address alice, bytes32 vaultId, int256 amount18) external {
        amount18 = bound(amount18, 1, type(int128).max);
        Float memory amount = Float(amount18, -18);

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

    /// forge-config: default.fuzz.runs = 10
    function testOrderBookDepositEvalWriteStateDifferentOwnersNamespaced(
        address alice,
        address bob,
        bytes32 vaultId,
        int256 amount18
    ) external {
        vm.assume(alice != bob);
        amount18 = bound(amount18, 1, type(int128).max);

        Float memory amount = Float(amount18, -18);

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

    /// forge-config: default.fuzz.runs = 10
    function testOrderDepositContext(address alice, bytes32 vaultId, int256 preDepositAmount18, int256 depositAmount18)
        external
    {
        preDepositAmount18 = bound(preDepositAmount18, 1, type(int128).max);
        depositAmount18 = bound(depositAmount18, 1, type(int128).max);

        Float memory preDepositAmount = Float(preDepositAmount18, -18);
        Float memory depositAmount = Float(depositAmount18, -18);

        checkDeposit(alice, vaultId, preDepositAmount, new bytes[](0), 0, 0);

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes[] memory evals = new bytes[](6);
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
                usingWordsFrom, ":ensure(equal-to(depositor() ", alice.toHexString(), ") \"depositor is alice\");"
            )
        );
        evals[2] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(deposit-token() ",
                address(iToken0).toHexString(),
                ") \"token is iToken0\");"
            )
        );
        evals[3] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(deposit-vault-id() ",
                uint256(vaultId).toHexString(),
                ") \"deposit vaultId is vaultId\");"
            )
        );
        evals[4] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(deposit-vault-balance() ",
                preDepositAmount.toDecimalString(),
                "e-6) \"vault balance is pre deposit\");"
            )
        );
        evals[5] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(deposit-amount() ",
                depositAmount.toDecimalString(),
                "e-6) \"amount is depositAmount\");"
            )
        );

        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));
        checkDeposit(alice, vaultId, depositAmount, evals, 0, 0);
    }

    /// A revert in the action prevents the deposit from being enacted.
    /// forge-config: default.fuzz.runs = 10
    function testDepositRevertInAction(address alice, bytes32 vaultId, uint256 amount18) external {
        vm.assume(amount18 > 0);
        vm.startPrank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), amount18),
            abi.encode(true)
        );

        Float memory amount = LibDecimalFloat.fromFixedDecimalLosslessMem(amount18, 18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"revert in action\");");

        TaskV2[] memory actions = evalsToActions(evals);

        assertTrue(iOrderbook.vaultBalance2(alice, address(iToken0), vaultId).eq(Float(0, 0)));

        vm.expectRevert("revert in action");
        iOrderbook.deposit3(address(iToken0), vaultId, amount, actions);

        assertTrue(iOrderbook.vaultBalance2(alice, address(iToken0), vaultId).eq(Float(0, 0)));
    }
}
