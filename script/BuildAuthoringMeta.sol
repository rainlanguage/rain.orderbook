// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {Script} from "forge-std/Script.sol";
import {LibOrderBookSubParser} from "../src/lib/LibOrderBookSubParser.sol";

/// @title OrderBook subparser Authoring Meta
/// @notice A script that writes the raw authoring meta out to file so it can be
/// wrapped in CBOR and emitted on metaboard.
contract BuildAuthoringMeta is Script {
    function run() external {
        vm.writeFileBinary("meta/OrderBookSubParserAuthoringMeta.rain.meta", LibOrderBookSubParser.authoringMetaV2());
    }
}
