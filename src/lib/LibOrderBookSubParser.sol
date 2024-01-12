// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {AuthoringMetaV2, Operand} from "rain.interpreter/interface/unstable/ISubParserV2.sol";

uint256 constant SUB_PARSER_WORD_PARSERS_LENGTH = 1;

/// @title LibOrderBookSubParser
library LibOrderBookSubParser {
    //slither-disable-next-line dead-code
    function authoringMetaV2() internal pure returns (bytes memory) {
        AuthoringMetaV2 memory lengthPlaceholder;
        AuthoringMetaV2[SUB_PARSER_WORD_PARSERS_LENGTH + 1] memory wordsFixed = [
            lengthPlaceholder,
            AuthoringMetaV2(
                "order-clearer",
                "The order clearer is the address that submitted the transaction that is causing the order to execute. This MAY be the counterparty, e.g. when an order is being taken directly, but it MAY NOT be the counterparty if a third party is clearing two orders against each other."
            )
        ];
        AuthoringMetaV2[] memory wordsDynamic;
        uint256 length = SUB_PARSER_WORD_PARSERS_LENGTH;
        assembly {
            wordsDynamic := wordsFixed
            mstore(wordsDynamic, length)
        }
        return abi.encode(wordsDynamic);
    }
}
