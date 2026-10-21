// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {
    IRaindexV6,
    OrderV4,
    IOV2,
    EvaluableV4,
    ClearConfigV2,
    SignedContextV1,
    TaskV2
} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rainlang-interface-0.2.13/src/interface/IInterpreterStoreV3.sol";
import {
    ReentrancyGuardReentrantCall,
    NotOrderOwner,
    TokenMismatch,
    MinimumIO,
    SameOwner,
    UnsupportedCalculateOutputs,
    NegativeVaultBalance,
    NegativeVaultBalanceChange,
    NegativePull,
    NegativePush,
    NegativeBounty,
    ClearZeroAmount
} from "src/error/ErrRaindexV6.sol";

/// @title RaindexV6ErrorSelectorTest
/// @notice Signature strings are the oracle for every error RaindexV6 reverts
/// with, both as declared and as the deployed artifact reverts with them.
contract RaindexV6ErrorSelectorTest is Test {
    IRaindexV6 internal raindex;

    function setUp() external {
        raindex = IRaindexV6(deployCode("RaindexV6.sol:RaindexV6"));
    }

    function testErrorSelectorsMatchSignatures() external pure {
        assertEq(ReentrancyGuardReentrantCall.selector, bytes4(keccak256("ReentrancyGuardReentrantCall()")));
        assertEq(NotOrderOwner.selector, bytes4(keccak256("NotOrderOwner(address)")));
        assertEq(TokenMismatch.selector, bytes4(keccak256("TokenMismatch()")));
        assertEq(MinimumIO.selector, bytes4(keccak256("MinimumIO(bytes32,bytes32)")));
        assertEq(SameOwner.selector, bytes4(keccak256("SameOwner()")));
        assertEq(UnsupportedCalculateOutputs.selector, bytes4(keccak256("UnsupportedCalculateOutputs(uint256)")));
        assertEq(NegativeVaultBalance.selector, bytes4(keccak256("NegativeVaultBalance(bytes32)")));
        assertEq(NegativeVaultBalanceChange.selector, bytes4(keccak256("NegativeVaultBalanceChange(bytes32)")));
        assertEq(NegativePull.selector, bytes4(keccak256("NegativePull()")));
        assertEq(NegativePush.selector, bytes4(keccak256("NegativePush()")));
        assertEq(NegativeBounty.selector, bytes4(keccak256("NegativeBounty()")));
        assertEq(ClearZeroAmount.selector, bytes4(keccak256("ClearZeroAmount()")));
    }

    function order(address owner, address inputToken, address outputToken) internal pure returns (OrderV4 memory) {
        IOV2[] memory inputs = new IOV2[](1);
        inputs[0] = IOV2(inputToken, bytes32(uint256(1)));
        IOV2[] memory outputs = new IOV2[](1);
        outputs[0] = IOV2(outputToken, bytes32(uint256(1)));
        return OrderV4({
            owner: owner,
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), ""),
            validInputs: inputs,
            validOutputs: outputs,
            nonce: bytes32(0)
        });
    }

    function clear(OrderV4 memory alice, OrderV4 memory bob) internal {
        raindex.clear3(
            alice,
            bob,
            ClearConfigV2(0, 0, 0, 0, bytes32(0), bytes32(0)),
            new SignedContextV1[](0),
            new SignedContextV1[](0)
        );
    }

    function testRemoveOrderNotOwnerRevertData(address owner, address sender) external {
        vm.assume(owner != sender);
        vm.prank(sender);
        vm.expectRevert(abi.encodeWithSelector(bytes4(keccak256("NotOrderOwner(address)")), owner));
        raindex.removeOrder3(order(owner, address(1), address(2)), new TaskV2[](0));
    }

    function testClearSameOwnerRevertData(address owner) external {
        vm.expectRevert(abi.encodeWithSelector(bytes4(keccak256("SameOwner()"))));
        clear(order(owner, address(1), address(2)), order(owner, address(2), address(1)));
    }

    function testClearTokenMismatchRevertData(address alice, address bob) external {
        vm.assume(alice != bob);
        vm.expectRevert(abi.encodeWithSelector(bytes4(keccak256("TokenMismatch()"))));
        clear(order(alice, address(1), address(2)), order(bob, address(1), address(2)));
    }
}
