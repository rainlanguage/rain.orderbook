// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script, console2} from "forge-std/Script.sol";
import {EvaluableV4, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV4.sol";
import {TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {OrderBookV6SubParser} from "../src/concrete/parser/OrderBookV6SubParser.sol";
import {GenericPoolOrderBookV6ArbOrderTaker} from "../src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol";
import {RouteProcessorOrderBookV6ArbOrderTaker} from "../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
import {GenericPoolOrderBookV6FlashBorrower} from "../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {OrderBookV6ArbConfig} from "../src/abstract/OrderBookV6ArbCommon.sol";
import {IMetaBoardV1_2} from "rain.metadata/interface/unstable/IMetaBoardV1_2.sol";
import {LibDescribedByMeta} from "rain.metadata/lib/LibDescribedByMeta.sol";
import {LibMetaBoardDeploy} from "rain.metadata/lib/deploy/LibMetaBoardDeploy.sol";
import {LibDecimalFloatDeploy} from "rain.math.float/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "../src/lib/deploy/LibOrderBookDeploy.sol";
import {CREATION_CODE as ORDERBOOK_CREATION_CODE} from "../src/generated/OrderBookV6.pointers.sol";
import {CREATION_CODE as SUB_PARSER_CREATION_CODE} from "../src/generated/OrderBookV6SubParser.pointers.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../src/lib/deploy/LibRouteProcessor4CreationCode.sol";

/// @dev Deploy only the OrderBookV6 (raindex) contract.
bytes32 constant DEPLOYMENT_SUITE_RAINDEX = keccak256("raindex");
/// @dev Deploy only the OrderBookV6SubParser contract.
bytes32 constant DEPLOYMENT_SUITE_SUBPARSER = keccak256("subparser");
/// @dev Deploy only the RouteProcessor4 contract.
bytes32 constant DEPLOYMENT_SUITE_ROUTE_PROCESSOR = keccak256("route-processor");
/// @dev Deploy only the arb contracts (order takers and flash borrowers).
bytes32 constant DEPLOYMENT_SUITE_ARB = keccak256("arb");

/// @title Deploy
/// @notice Foundry script that deploys orderbook contracts. Controlled by the
/// `DEPLOYMENT_SUITE` env var to select which subset to deploy, and
/// `DEPLOYMENT_KEY` for the deployer private key.
contract Deploy is Script {
    /// @dev Tracks on-chain code hashes of dependencies per network, passed to
    /// `LibRainDeploy.deployAndBroadcast` to verify dependency integrity.
    mapping(string => mapping(address => bytes32)) internal sDepCodeHashes;

    /// @dev Entry point. Reads `DEPLOYMENT_KEY` and `DEPLOYMENT_SUITE` from env
    /// then deploys the selected contracts via `LibRainDeploy`.
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        string memory suiteString = vm.envString("DEPLOYMENT_SUITE");
        bytes32 suite = keccak256(bytes(suiteString));

        if (suite == DEPLOYMENT_SUITE_RAINDEX) {
            console2.log("Deploying OrderBookV6...");
            address[] memory deps = new address[](3);
            deps[0] = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
            deps[1] = address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT);
            deps[2] = LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS;
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                ORDERBOOK_CREATION_CODE,
                "src/concrete/ob/OrderBookV6.sol:OrderBookV6",
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else if (suite == DEPLOYMENT_SUITE_SUBPARSER) {
            console2.log("Deploying OrderBookV6SubParser...");
            address[] memory deps = new address[](3);
            deps[0] = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
            deps[1] = address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT);
            deps[2] = LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS;
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                SUB_PARSER_CREATION_CODE,
                "src/concrete/parser/OrderBookV6SubParser.sol:OrderBookV6SubParser",
                LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS,
                LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
            IMetaBoardV1_2 metaboard = IMetaBoardV1_2(LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS);
            vm.startBroadcast(deployerPrivateKey);
            bytes memory subParserDescribedByMeta = vm.readFileBinary("meta/OrderBookV6SubParser.rain.meta");
            LibDescribedByMeta.emitForDescribedAddress(
                metaboard,
                OrderBookV6SubParser(LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS),
                subParserDescribedByMeta
            );
            vm.stopBroadcast();
        } else if (suite == DEPLOYMENT_SUITE_ROUTE_PROCESSOR) {
            console2.log("Deploying RouteProcessor4...");
            address[] memory deps = new address[](0);
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                ROUTE_PROCESSOR_4_CREATION_CODE,
                "RouteProcessor4",
                LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS,
                LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else if (suite == DEPLOYMENT_SUITE_ARB) {
            vm.startBroadcast(deployerPrivateKey);
            OrderBookV6ArbConfig memory arbConfig = OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                })
            );
            new GenericPoolOrderBookV6ArbOrderTaker(arbConfig);
            new RouteProcessorOrderBookV6ArbOrderTaker(arbConfig);
            new GenericPoolOrderBookV6FlashBorrower(arbConfig);
            vm.stopBroadcast();
        } else {
            revert("Unknown deployment suite");
        }
    }
}
