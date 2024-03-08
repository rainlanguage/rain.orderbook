// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {AuthoringMetaV2, Operand} from "rain.interpreter.interface/interface/ISubParserV2.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";
import {LibSubParse} from "rain.interpreter/lib/parse/LibSubParse.sol";
import {
    CONTEXT_BASE_COLUMN,
    CONTEXT_BASE_ROW_SENDER,
    CONTEXT_BASE_ROW_CALLING_CONTRACT,
    CONTEXT_BASE_ROWS,
    CONTEXT_COLUMNS,
    CONTEXT_VAULT_OUTPUTS_COLUMN,
    CONTEXT_VAULT_INPUTS_COLUMN,
    CONTEXT_CALCULATIONS_COLUMN,
    CONTEXT_CALLING_CONTEXT_COLUMN,
    CONTEXT_VAULT_IO_BALANCE_DIFF,
    CONTEXT_VAULT_IO_BALANCE_BEFORE,
    CONTEXT_VAULT_IO_VAULT_ID,
    CONTEXT_VAULT_IO_TOKEN_DECIMALS,
    CONTEXT_VAULT_IO_TOKEN,
    CONTEXT_VAULT_IO_ROWS,
    CONTEXT_CALCULATIONS_ROW_IO_RATIO,
    CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT,
    CONTEXT_CALCULATIONS_ROWS,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER,
    CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH,
    CONTEXT_CALLING_CONTEXT_ROWS,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW,
    CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS,
    CONTEXT_SIGNED_CONTEXT_START_COLUMN,
    CONTEXT_SIGNED_CONTEXT_START_ROW,
    CONTEXT_SIGNED_CONTEXT_START_ROWS
} from "./LibOrderBook.sol";

uint256 constant SUB_PARSER_WORD_PARSERS_LENGTH = 2;

bytes constant WORD_ORDER_CLEARER = "order-clearer";
bytes constant WORD_ORDERBOOK = "orderbook";
bytes constant WORD_ORDER_HASH = "order-hash";
bytes constant WORD_ORDER_OWNER = "order-owner";
bytes constant WORD_ORDER_COUNTERPARTY = "order-counterparty";
bytes constant WORD_CALCULATED_MAX_OUTPUT = "calculated-max-output";
bytes constant WORD_CALCULATED_IO_RATIO = "calculated-io-ratio";
bytes constant WORD_INPUT_TOKEN = "input-token";
bytes constant WORD_INPUT_TOKEN_DECIMALS = "input-token-decimals";
bytes constant WORD_INPUT_VAULT_ID = "input-vault-id";
bytes constant WORD_INPUT_VAULT_BALANCE_BEFORE = "input-vault-balance-before";
bytes constant WORD_INPUT_VAULT_BALANCE_INCREASE = "input-vault-balance-increase";
bytes constant WORD_OUTPUT_TOKEN = "output-token";
bytes constant WORD_OUTPUT_TOKEN_DECIMALS = "output-token-decimals";
bytes constant WORD_OUTPUT_VAULT_ID = "output-vault-id";
bytes constant WORD_OUTPUT_VAULT_BALANCE_BEFORE = "output-vault-balance-before";
bytes constant WORD_OUTPUT_VAULT_BALANCE_DECREASE = "output-vault-balance-decrease";

/// @title LibOrderBookSubParser
library LibOrderBookSubParser {
    using LibUint256Matrix for uint256[][];

    function subParserSender(uint256, uint256, Operand) internal pure returns (bool, bytes memory, uint256[] memory) {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_BASE_COLUMN, CONTEXT_BASE_ROW_SENDER);
    }

    function subParserCallingContract(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_BASE_COLUMN, CONTEXT_BASE_ROW_CALLING_CONTRACT);
    }

    function subParserOrderHash(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH);
    }

    function subParserOrderOwner(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER);
    }

    function subParserOrderCounterparty(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return
            LibSubParse.subParserContext(CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY);
    }

    function subParserMaxOutput(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_CALCULATIONS_COLUMN, CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT);
    }

    function subParserIORatio(uint256, uint256, Operand) internal pure returns (bool, bytes memory, uint256[] memory) {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_CALCULATIONS_COLUMN, CONTEXT_CALCULATIONS_ROW_IO_RATIO);
    }

    function subParserInputToken(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN);
    }

    function subParserInputTokenDecimals(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN_DECIMALS);
    }

    function subParserInputVaultId(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_VAULT_ID);
    }

    function subParserInputBalanceBefore(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_BEFORE);
    }

    function subParserInputBalanceDiff(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_DIFF);
    }

    function subParserOutputToken(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN);
    }

    function subParserOutputTokenDecimals(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN_DECIMALS);
    }

    function subParserOutputVaultId(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_VAULT_ID);
    }

    function subParserOutputBalanceBefore(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_BEFORE);
    }

    function subParserOutputBalanceDiff(uint256, uint256, Operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_DIFF);
    }

    function subParserSigners(uint256, uint256, Operand operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN, Operand.unwrap(operand));
    }

    function subParserSignedContext(uint256, uint256, Operand operand)
        internal
        pure
        returns (bool, bytes memory, uint256[] memory)
    {
        uint256 column = Operand.unwrap(operand) & 0xFF;
        uint256 row = (Operand.unwrap(operand) >> 8) & 0xFF;
        //slither-disable-next-line unused-return
        return LibSubParse.subParserContext(CONTEXT_SIGNED_CONTEXT_START_COLUMN + column, row);
    }

    //slither-disable-next-line dead-code
    function authoringMetaV2() internal pure returns (bytes memory) {
        // Add 2 for the signed context signers and signed context start columns.
        AuthoringMetaV2[][] memory meta = new AuthoringMetaV2[][](CONTEXT_COLUMNS + 2);

        AuthoringMetaV2[] memory contextBaseMeta = new AuthoringMetaV2[](CONTEXT_BASE_ROWS);
        contextBaseMeta[CONTEXT_BASE_ROW_SENDER] = AuthoringMetaV2(
            bytes32(WORD_ORDER_CLEARER),
            "The order clearer is the address that submitted the transaction that is causing the order to execute. This MAY be the counterparty, e.g. when an order is being taken directly, but it MAY NOT be the counterparty if a third party is clearing two orders against each other."
        );
        contextBaseMeta[CONTEXT_BASE_ROW_CALLING_CONTRACT] =
            AuthoringMetaV2(bytes32(WORD_ORDERBOOK), "The address of the orderbook that the order is being run on.");

        AuthoringMetaV2[] memory contextCallingContextMeta = new AuthoringMetaV2[](CONTEXT_CALLING_CONTEXT_ROWS);
        contextCallingContextMeta[CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH] =
            AuthoringMetaV2(bytes32(WORD_ORDER_HASH), "The hash of the order that is being cleared.");
        contextCallingContextMeta[CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER] =
            AuthoringMetaV2(bytes32(WORD_ORDER_OWNER), "The address of the order owner.");
        contextCallingContextMeta[CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY] = AuthoringMetaV2(
            bytes32(WORD_ORDER_COUNTERPARTY),
            "The address of the owner of the counterparty order. Will be the order taker if there is no counterparty order."
        );

        AuthoringMetaV2[] memory contextCalculationsMeta = new AuthoringMetaV2[](CONTEXT_CALCULATIONS_ROWS);
        contextCalculationsMeta[CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT] = AuthoringMetaV2(
            bytes32(WORD_CALCULATED_MAX_OUTPUT),
            "The maximum output of the order, i.e. the maximum amount of the output token that the order will send. This is normalized to 18 decimal fixed point regardless of the decimals of the underlying token. This is 0 before calculations have been run."
        );
        contextCalculationsMeta[CONTEXT_CALCULATIONS_ROW_IO_RATIO] = AuthoringMetaV2(
            bytes32(WORD_CALCULATED_IO_RATIO),
            "The ratio of the input to output token, i.e. the amount of the input token that the order will receive for each unit of the output token that it sends. This is normalized to 18 decimal fixed point regardless of the decimals of the underlying tokens. This is 0 before calculations have been run."
        );

        AuthoringMetaV2[] memory contextVaultInputsMeta = new AuthoringMetaV2[](CONTEXT_VAULT_IO_ROWS);
        contextVaultInputsMeta[CONTEXT_VAULT_IO_TOKEN] =
            AuthoringMetaV2(bytes32(WORD_INPUT_TOKEN), "The address of the input token for the vault input.");
        contextVaultInputsMeta[CONTEXT_VAULT_IO_TOKEN_DECIMALS] =
            AuthoringMetaV2(bytes32(WORD_INPUT_TOKEN_DECIMALS), "The decimals of the input token for the vault input.");
        contextVaultInputsMeta[CONTEXT_VAULT_IO_VAULT_ID] = AuthoringMetaV2(
            bytes32(WORD_INPUT_VAULT_ID), "The ID of the input vault that incoming tokens are received into."
        );
        contextVaultInputsMeta[CONTEXT_VAULT_IO_BALANCE_BEFORE] = AuthoringMetaV2(
            bytes32(WORD_INPUT_VAULT_BALANCE_BEFORE), "The balance of the input vault before the order is cleared."
        );
        contextVaultInputsMeta[CONTEXT_VAULT_IO_BALANCE_DIFF] = AuthoringMetaV2(
            bytes32(WORD_INPUT_VAULT_BALANCE_INCREASE),
            "The difference in the balance of the input vault after the order is cleared. This is always positive so it must be added to the input balance before to get the final vault balance. This is 0 before calculations have been run."
        );

        AuthoringMetaV2[] memory contextVaultOutputsMeta = new AuthoringMetaV2[](CONTEXT_VAULT_IO_ROWS);
        contextVaultOutputsMeta[CONTEXT_VAULT_IO_TOKEN] =
            AuthoringMetaV2(bytes32(WORD_OUTPUT_TOKEN), "The address of the output token for the vault output.");
        contextVaultOutputsMeta[CONTEXT_VAULT_IO_TOKEN_DECIMALS] = AuthoringMetaV2(
            bytes32(WORD_OUTPUT_TOKEN_DECIMALS), "The decimals of the output token for the vault output."
        );
        contextVaultOutputsMeta[CONTEXT_VAULT_IO_VAULT_ID] = AuthoringMetaV2(
            bytes32(WORD_OUTPUT_VAULT_ID), "The ID of the output vault that outgoing tokens are sent from."
        );
        contextVaultOutputsMeta[CONTEXT_VAULT_IO_BALANCE_BEFORE] = AuthoringMetaV2(
            bytes32(WORD_OUTPUT_VAULT_BALANCE_BEFORE), "The balance of the output vault before the order is cleared."
        );
        contextVaultOutputsMeta[CONTEXT_VAULT_IO_BALANCE_DIFF] = AuthoringMetaV2(
            bytes32(WORD_OUTPUT_VAULT_BALANCE_DECREASE),
            "The difference in the balance of the output vault after the order is cleared. This is always positive so it must be subtracted from the output balance before to get the final vault balance. This is 0 before calculations have been run."
        );

        AuthoringMetaV2[] memory contextSignersMeta = new AuthoringMetaV2[](CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS);
        contextSignersMeta[CONTEXT_SIGNED_CONTEXT_SIGNERS_ROW] = AuthoringMetaV2(
            bytes32("signer"),
            "The addresses of the signers of the signed context. The indexes of the signers matches the column they signed in the signed context grid."
        );

        AuthoringMetaV2[] memory contextSignedMeta = new AuthoringMetaV2[](CONTEXT_SIGNED_CONTEXT_START_ROWS);
        contextSignedMeta[CONTEXT_SIGNED_CONTEXT_START_ROW] = AuthoringMetaV2(
            bytes32("signed-context"),
            "Signed context is provided by the order clearer/taker and can be signed by anyone. Orderbook will check the signature, but the expression author much authorize the signer's public key."
        );

        meta[CONTEXT_BASE_COLUMN] = contextBaseMeta;
        meta[CONTEXT_CALLING_CONTEXT_COLUMN] = contextCallingContextMeta;
        meta[CONTEXT_CALCULATIONS_COLUMN] = contextCalculationsMeta;
        meta[CONTEXT_VAULT_INPUTS_COLUMN] = contextVaultInputsMeta;
        meta[CONTEXT_VAULT_OUTPUTS_COLUMN] = contextVaultOutputsMeta;
        meta[CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN] = contextSignersMeta;
        meta[CONTEXT_SIGNED_CONTEXT_START_COLUMN] = contextSignedMeta;

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
