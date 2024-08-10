// THIS FILE IS AUTOGENERATED BY ./script/BuildPointers.sol

// This file is committed to the repository because there is a circular
// dependency between the contract and its pointers file. The contract
// needs the pointers file to exist so that it can compile, and the pointers
// file needs the contract to exist so that it can be compiled.

// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

/// @dev Hash of the known bytecode.
bytes32 constant BYTECODE_HASH = bytes32(0x88ef869533ea00f35b7537bb4157ea4965947f21bcbe5dea5654cbfd8313a397);

/// @dev The hash of the meta that describes the contract.
bytes32 constant DESCRIBED_BY_META_HASH = bytes32(0x37096fdde7b025c34cf7d2774f97f0fbb7711051fe45d959b7d34acadc31e4ee);

/// @dev The parse meta that is used to lookup word definitions.
/// The structure of the parse meta is:
/// - 1 byte: The depth of the bloom filters
/// - 1 byte: The hashing seed
/// - The bloom filters, each is 32 bytes long, one for each build depth.
/// - All the items for each word, each is 4 bytes long. Each item's first byte
///   is its opcode index, the remaining 3 bytes are the word fingerprint.
/// To do a lookup, the word is hashed with the seed, then the first byte of the
/// hash is compared against the bloom filter. If there is a hit then we count
/// the number of 1 bits in the bloom filter up to this item's 1 bit. We then
/// treat this a the index of the item in the items array. We then compare the
/// word fingerprint against the fingerprint of the item at this index. If the
/// fingerprints equal then we have a match, else we increment the seed and try
/// again with the next bloom filter, offsetting all the indexes by the total
/// bit count of the previous bloom filter. If we reach the end of the bloom
/// filters then we have a miss.
bytes constant PARSE_META =
    hex"01004800040040420206180010a00040003806204008040820840086010100000030088de69a165139fb02c9be1f14361d851dc13efa18bc2b16131296c8116682f50b6f6a660584c8d41c02582b1ad22c2f06bbcde61ea8967915b4f4091283156f0109ac301087b0c70398cd201f611967211923d122dd2b8317c02b171b3926a52035ccb90ea9bcef19d276fe0a865655075e0bc300d3b4e80f8316290de78f2e0c9fc5d509a7e6560427db4a";

/// @dev The build depth of the parser meta.

uint8 constant PARSE_META_BUILD_DEPTH = 1;

/// @dev The function pointers for the sub parser functions that produce the
/// bytecode that this contract knows about. This is both constructing the subParser
/// bytecode that dials back into this contract at eval time, and mapping
/// to things that happen entirely on the interpreter such as well known
/// constants and references to the context grid.
bytes constant SUB_PARSER_WORD_PARSERS =
    hex"1962197f198e199d19ac19bc19cb19db19ea19fa1a0a1a191a291a381a481a581a681a771a861962198e199d19ac1abb1acb1adb1962198e199d19ac1abb1acb1adb1aeb1afb";

/// @dev Every two bytes is a function pointer for an operand handler.
/// These positional indexes all map to the same indexes looked up in the parse
/// meta.
bytes constant OPERAND_HANDLER_FUNCTION_POINTERS =
    hex"1c181c181c181c181c181c181c181c181c181c181c181c181c181c181c181c181c181c5a1ce71c181c181c181c181c181c181c181c181c181c181c181c181c181c181c181c18";

/// @dev Every two bytes is a function pointer for a literal parser.
/// Literal dispatches are determined by the first byte(s) of the literal
/// rather than a full word lookup, and are done with simple conditional
/// jumps as the possibilities are limited compared to the number of words we
/// have.
bytes constant LITERAL_PARSER_FUNCTION_POINTERS = hex"";
