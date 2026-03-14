// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {Vm} from "forge-std/Vm.sol";

import {
    BYTECODE_HASH as ORDERBOOK_HASH,
    DEPLOYED_ADDRESS as ORDERBOOK_ADDR,
    RUNTIME_CODE as ORDERBOOK_RUNTIME_CODE
} from "../../generated/OrderBookV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE
} from "../../generated/OrderBookV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH,
    DEPLOYED_ADDRESS as ROUTE_PROCESSOR_ADDR,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_CODE
} from "../../generated/RouteProcessor4.pointers.sol";

/// @title LibOrderBookDeploy
/// @notice A library containing the deployed address and code hash of the
/// OrderBook contracts when deployed with the rain standard zoltu deployer.
/// This allows idempotent deployments against precommitted addresses and hashes
/// that can be easily verified automatically in tests and scripts rather than
/// relying on registries or manual verification.
library LibOrderBookDeploy {
    /// The address of the `OrderBookV6` contract when deployed with the rain
    /// standard zoltu deployer.
    address constant ORDERBOOK_DEPLOYED_ADDRESS = ORDERBOOK_ADDR;

    /// The code hash of the `OrderBookV6` contract when deployed with the rain
    /// standard zoltu deployer.
    bytes32 constant ORDERBOOK_DEPLOYED_CODEHASH = ORDERBOOK_HASH;

    /// The address of the `OrderBookV6SubParser` contract when deployed with
    /// the rain standard zoltu deployer.
    address constant SUB_PARSER_DEPLOYED_ADDRESS = SUB_PARSER_ADDR;

    /// The code hash of the `OrderBookV6SubParser` contract when deployed with
    /// the rain standard zoltu deployer.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH = SUB_PARSER_HASH;

    /// The address of the `RouteProcessor4` contract when deployed with the
    /// rain standard zoltu deployer.
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS = ROUTE_PROCESSOR_ADDR;

    /// The code hash of the `RouteProcessor4` contract when deployed with the
    /// rain standard zoltu deployer.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH = ROUTE_PROCESSOR_HASH;

    /// @notice Etches the runtime bytecode of the orderbook and sub parser at
    /// their expected deterministic addresses. Skips any contract whose
    /// codehash already matches.
    /// @param vm The Forge `Vm` cheatcode interface.
    function etchOrderBook(Vm vm) internal {
        if (ORDERBOOK_DEPLOYED_CODEHASH != ORDERBOOK_DEPLOYED_ADDRESS.codehash) {
            vm.etch(ORDERBOOK_DEPLOYED_ADDRESS, ORDERBOOK_RUNTIME_CODE);
        }
        if (SUB_PARSER_DEPLOYED_CODEHASH != SUB_PARSER_DEPLOYED_ADDRESS.codehash) {
            vm.etch(SUB_PARSER_DEPLOYED_ADDRESS, SUB_PARSER_RUNTIME_CODE);
        }
        if (ROUTE_PROCESSOR_DEPLOYED_CODEHASH != ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash) {
            vm.etch(ROUTE_PROCESSOR_DEPLOYED_ADDRESS, ROUTE_PROCESSOR_RUNTIME_CODE);
        }
    }
}
