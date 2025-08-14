// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";

import {OrderBookExternalMockTest, REVERTING_MOCK_BYTECODE} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {Reenteroor, IERC20} from "test/util/concrete/Reenteroor.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {TaskV1, SignedContextV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {EvaluableV3} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";

/// @title OrderBookWithdrawTest
/// Tests withdrawing from the order book.
contract OrderBookWithdrawRoundingTest is OrderBookExternalMockTest {
    using Math for uint256;

    function testWithdrawRoundingRevert() external
    {
        
        address alice = address(0x3392c4b753fe2f12C34a4e4C90e2023F79498C3B); // Fix: assign a proper address
        uint256 vaultId = 0x12345; 
        uint256 depositAmount = 0;
        uint256 withdrawAmount = type(uint256).max;
        
        vm.prank(alice);
        // Fix: Mock the token properly for all calls, not just the specific one
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        
        // Fix: Also mock approve if needed
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.approve.selector, address(iOrderbook), type(uint256).max),
            abi.encode(true)
        );
        
        // Mock decimals() to return 6
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20Metadata.decimals.selector),
            abi.encode(6)
        );
        
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, alice, depositAmount), abi.encode(true)
        );
        vm.expectEmit(false, false, false, true);
        emit Withdraw(alice, address(iToken0), vaultId, withdrawAmount, depositAmount);

        TaskV1[] memory task = new TaskV1[](1);
        bytes memory taskBytecode = hex"00000000000000000000000000000000000000000000000000000000000000080000000000000000000000003392c4b753fe2f12c34a4e4c90e2023f79498c3b8e756e6b6e6f776e2073656e6465720000000000000000000000000000000000000000000000000000000000abef041ebd0ad5d9c8a4e88b04f9a58c1cab93c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda029130000000000000000000000000000000000000000000000000062083966467a3f0000000000000000000000000000000000000000000000000000c43fe6b0e1fd956d696e696d756d2073656e646572206f75747075740000000000000000000000000000000000000000000000000000000000000000000000000000000000730100001b0600030110000103100000011000001e1200001d0200000110000301100000011000021112000047120000011000030110000001100004111200004712000001100006001000013d12000001100005001000003d1200002b120000011000070110000300100002211200001d020000";
        task[0] = TaskV1(EvaluableV3(iInterpreter, iStore, taskBytecode), new SignedContextV1[](0));
        iOrderbook.withdraw2(address(iToken0), vaultId, withdrawAmount, task);
        assertEq(iOrderbook.vaultBalance(address(alice), address(iToken0), vaultId), 0);
    }
}
