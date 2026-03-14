// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployProdTest
/// @notice Forks each supported network and verifies that both orderbook
/// contracts are deployed at the expected addresses with the expected codehash.
contract LibOrderBookDeployProdTest is Test {
    function _checkAllContracts() internal view {
        assertTrue(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.code.length > 0, "OrderBookV6 not deployed");
        assertEq(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);

        assertTrue(LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.code.length > 0, "SubParser not deployed");
        assertEq(
            LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );

        assertTrue(LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.code.length > 0, "RouteProcessor4 not deployed");
        assertEq(
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash,
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );
    }

    /// Both contracts MUST be deployed on Arbitrum.
    function testProdDeployArbitrum() external {
        vm.createSelectFork(LibRainDeploy.ARBITRUM_ONE);
        _checkAllContracts();
    }

    /// Both contracts MUST be deployed on Base.
    function testProdDeployBase() external {
        vm.createSelectFork(LibRainDeploy.BASE);
        _checkAllContracts();
    }

    /// Both contracts MUST be deployed on Base Sepolia.
    function testProdDeployBaseSepolia() external {
        vm.createSelectFork(LibRainDeploy.BASE_SEPOLIA);
        _checkAllContracts();
    }

    /// Both contracts MUST be deployed on Flare.
    function testProdDeployFlare() external {
        vm.createSelectFork(LibRainDeploy.FLARE);
        _checkAllContracts();
    }

    /// Both contracts MUST be deployed on Polygon.
    function testProdDeployPolygon() external {
        vm.createSelectFork(LibRainDeploy.POLYGON);
        _checkAllContracts();
    }
}
