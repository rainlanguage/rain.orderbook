// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script} from "forge-std/Script.sol";
import {LibCodeGen} from "rain.sol.codegen/lib/LibCodeGen.sol";
import {LibFs} from "rain.sol.codegen/lib/LibFs.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {LibOrderBookSubParser, EXTERN_PARSE_META_BUILD_DEPTH} from "src/lib/LibOrderBookSubParser.sol";
import {LibGenParseMeta} from "rain.interpreter.interface/lib/codegen/LibGenParseMeta.sol";

contract BuildPointers is Script {
    function buildOrderBookSubParserPointers() internal {
        OrderBookSubParser subParser = new OrderBookSubParser();

        string memory name = "OrderBookSubParser";

        LibFs.buildFileForContract(
            vm,
            address(subParser),
            name,
            string.concat(
                LibCodeGen.describedByMetaHashConstantString(vm, name),
                LibGenParseMeta.parseMetaConstantString(
                    vm, LibOrderBookSubParser.authoringMetaV2(), EXTERN_PARSE_META_BUILD_DEPTH
                ),
                LibCodeGen.subParserWordParsersConstantString(vm, subParser),
                LibCodeGen.operandHandlerFunctionPointersConstantString(vm, subParser),
                LibCodeGen.literalParserFunctionPointersConstantString(vm, subParser)
            )
        );
    }

    function run() external {
        buildOrderBookSubParserPointers();
    }
}
