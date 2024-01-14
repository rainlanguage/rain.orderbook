// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {AuthoringMetaV2, Operand} from "rain.interpreter/interface/unstable/ISubParserV2.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";
import {
    CONTEXT_BASE_COLUMN,
    CONTEXT_BASE_ROW_SENDER,
    CONTEXT_BASE_ROW_CALLING_CONTRACT,
    CONTEXT_BASE_ROWS,
    CONTEXT_COLUMNS
} from "./LibOrderBook.sol";

uint256 constant SUB_PARSER_WORD_PARSERS_LENGTH = 2;

/// @title LibOrderBookSubParser
library LibOrderBookSubParser {
    using LibUint256Matrix for uint256[][];

    //slither-disable-next-line dead-code
    function authoringMetaV2() internal pure returns (bytes memory) {
        AuthoringMetaV2[][] memory meta = new AuthoringMetaV2[][](CONTEXT_COLUMNS);

        AuthoringMetaV2[] memory contextBaseMeta = new AuthoringMetaV2[](CONTEXT_BASE_ROWS);
        contextBaseMeta[CONTEXT_BASE_ROW_SENDER] = AuthoringMetaV2(
            "order-clearer",
            "The order clearer is the address that submitted the transaction that is causing the order to execute. This MAY be the counterparty, e.g. when an order is being taken directly, but it MAY NOT be the counterparty if a third party is clearing two orders against each other."
        );
        contextBaseMeta[CONTEXT_BASE_ROW_CALLING_CONTRACT] =
            AuthoringMetaV2("orderbook", "The address of the orderbook that the order is being run on.");

        meta[CONTEXT_BASE_COLUMN] = contextBaseMeta;

        uint256[][] memory metaUint256;
        assembly {
            metaUint256 := meta
        }
        uint256[] memory metaUint256Flattened = metaUint256.flatten();
        AuthoringMetaV2[] memory metaFlattened;
        assembly {
            metaFlattened := metaUint256Flattened
        }

        return abi.encode(metaFlattened);
    }
}
