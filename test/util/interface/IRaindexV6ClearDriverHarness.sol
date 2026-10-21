// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {IRaindexV6, Float} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";

interface IRaindexV6ClearDriverHarness is IRaindexV6 {
    function setOrderLive(bytes32 orderHash) external;

    function seedVault(address owner, address token, bytes32 vaultId, Float amount) external;
}
