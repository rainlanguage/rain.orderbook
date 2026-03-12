// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibMetaBoardDeploy} from "rain.metadata/lib/deploy/LibMetaBoardDeploy.sol";

contract LibMetaBoardDeployNetworksTest is Test {
    function testNetworksJsonMatchesZoltu() external view {
        string memory json = vm.readFile("lib/rain.interpreter/lib/rain.metadata/subgraph/networks.json");

        address expected = LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS;

        assertEq(vm.parseJsonAddress(json, "$.matic.metaboard0.address"), expected, "matic");
        assertEq(vm.parseJsonAddress(json, '$.["arbitrum-one"].metaboard0.address'), expected, "arbitrum-one");
        assertEq(vm.parseJsonAddress(json, "$.base.metaboard0.address"), expected, "base");
        assertEq(vm.parseJsonAddress(json, "$.mainnet.metaboard0.address"), expected, "mainnet");
        assertEq(vm.parseJsonAddress(json, '$.["berachain-mainnet"].metaboard0.address'), expected, "berachain-mainnet");
        assertEq(vm.parseJsonAddress(json, "$.sonic.metaboard0.address"), expected, "sonic");
    }
}
