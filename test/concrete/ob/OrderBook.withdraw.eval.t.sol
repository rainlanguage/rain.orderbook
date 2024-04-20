// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {OrderConfigV3, EvaluableV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";

contract OrderBookWithdrawEvalTest is OrderBookExternalRealTest {
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

    function checkWithdraw(uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount, bytes[] memory evalStrings, uint256 expectedReads, uint256 expectedWrites) internal {
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new EvaluableV3[](0));

        EvaluableV3[] memory evals = new EvaluableV3[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            evals[i] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2(evalStrings[i]));
        }
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(this), withdrawAmount),
            abi.encode(true)
        );
        vm.record();
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, evals);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == expectedReads);
        assert(writes.length == expectedWrites);
    }

    function testOrderBookWithdrawEvalEmptyNoop(uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        checkWithdraw(vaultId, depositAmount, withdrawAmount, new bytes[](0), 0, 0);
    }

    function testOrderBookWithdrawEvalOneStateless(uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        vm.record();
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new EvaluableV3[](0));

        EvaluableV3[] memory evals = new EvaluableV3[](1);
        evals[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2("_:1;"));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(this), withdrawAmount),
            abi.encode(true)
        );
        vm.record();
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, evals);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == 0);
        assert(writes.length == 0);
    }

    function testOrderBookWithdrawEvalOneReadState(uint256 vaultId, uint256 depositAmount, uint256 withdrawAmount) external {
        depositAmount = bound(depositAmount, 1, type(uint128).max);
        withdrawAmount = bound(withdrawAmount, 1, depositAmount);

        vm.record();
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(iToken0), vaultId, depositAmount, new EvaluableV3[](0));

        EvaluableV3[] memory evals = new EvaluableV3[](1);
        evals[0] = EvaluableV3(iInterpreter, iStore, iParserV2.parse2("_:get(0);"));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(this), withdrawAmount),
            abi.encode(true)
        );
        vm.record();
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, evals);
        checkReentrancyRW();
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
        assert(reads.length == 1);
        assert(reads[0] == bytes32(uint256(1)));
        assert(writes.length == 0);
    }
}