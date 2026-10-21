// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DeployCandidate} from "../../src/abstract/RainDeploySuitesBase.sol";
import {RaindexDeploySuites} from "../../src/abstract/RaindexDeploySuites.sol";

/// @title RaindexDeploySuitesTest
/// @notice Properties of the deploy declaration that `rain-deploy` ships no
/// assertion for.
contract RaindexDeploySuitesTest is Test, RaindexDeploySuites {
    uint256 constant EIP170_MAX_RUNTIME_CODE_SIZE = 0x6000;

    /// PROPERTY: every candidate's runtime code fits EIP-170.
    function testCandidateRuntimeCodeFitsEip170() external pure {
        DeployCandidate[] memory candidates = checkedCandidateSuites();
        for (uint256 i = 0; i < candidates.length; i++) {
            assertLe(
                candidates[i].snapshot.storedRuntimeCode.length,
                EIP170_MAX_RUNTIME_CODE_SIZE,
                string.concat("runtime code over EIP-170: ", candidates[i].snapshot.suite)
            );
        }
    }
}
