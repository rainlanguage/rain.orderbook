// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

/// Thrown when a TOFU decimals read fails during deposit.
/// @param token The token that failed to read decimals.
/// @param tofuOutcome The outcome of the TOFU read.
error TokenDecimalsReadFailure(address token, TOFUOutcome tofuOutcome);

/// Encodes the token's decimals for a token. Includes a bool to indicate if
/// the token's decimals have been read from the external contract before. This
/// guards against the default `0` value for unset storage data being
/// misinterpreted as a valid token decimal value `0`.
/// @param initialized True if the token's decimals have been read from the
/// external contract before.
/// @param tokenDecimals The token's decimals.
struct TOFUTokenDecimals {
    bool initialized;
    uint8 tokenDecimals;
}

enum TOFUOutcome {
    /// Token's decimals have not been read from the external contract before.
    Initial,
    /// Token's decimals are consistent with the stored value.
    Consistent,
    /// Token's decimals are inconsistent with the stored value.
    Inconsistent,
    /// Token's decimals could not be read from the external contract.
    ReadFailure
}

library LibTOFUTokenDecimals {
    function decimalsForTokenReadOnly(mapping(address => TOFUTokenDecimals) storage sTOFUTokenDecimals, address token)
        internal
        view
        returns (TOFUOutcome, uint8)
    {
        TOFUTokenDecimals memory tofuTokenDecimals = sTOFUTokenDecimals[token];

        // The default solidity try/catch logic will error if the return is a
        // success but fails to deserialize to the target type. We need to handle
        // all errors as read failures so that the calling context can decide
        // whether to revert the current transaction or continue with the stored
        // value. E.g. withdrawals will prefer to continue than trap funds, and
        // deposits will prefer to revert and prevent new funds entering the
        // DEX.
        (bool success, bytes memory returnData) = token.staticcall(abi.encodeWithSignature("decimals()"));
        if (!success || returnData.length != 0x20) {
            return (TOFUOutcome.ReadFailure, tofuTokenDecimals.tokenDecimals);
        }

        uint256 decodedDecimals = abi.decode(returnData, (uint256));
        if (decodedDecimals > type(uint8).max) {
            return (TOFUOutcome.ReadFailure, tofuTokenDecimals.tokenDecimals);
        }
        uint8 readDecimals = uint8(decodedDecimals);

        if (!tofuTokenDecimals.initialized) {
            return (TOFUOutcome.Initial, readDecimals);
        } else {
            return (
                readDecimals == tofuTokenDecimals.tokenDecimals ? TOFUOutcome.Consistent : TOFUOutcome.Inconsistent,
                tofuTokenDecimals.tokenDecimals
            );
        }
    }

    /// Trust on first use (TOFU) token decimals.
    /// The first time we read the decimals from a token we store them in a
    /// mapping. If the token's decimals change we will always use the stored
    /// value. This is because the token's decimals could technically change and
    /// are NOT intended for onchain use as they are optional, but we're doing
    /// it anyway to convert to floating point numbers.
    ///
    /// If we have nothing stored we read from the token, store and return it
    /// with TOFUOUTCOME.Consistent.
    ///
    /// If the call to `decimals` is not a success that deserializes cleanly to
    /// a `uint8` we return the stored value and TOFUOUTCOME.ReadFailure.
    ///
    /// If the stored value is inconsistent with the token's decimals we return
    /// the stored value and TOFUOUTCOME.Inconsistent.
    ///
    /// @return True if the token's decimals are consistent with the stored
    /// value.
    /// @return The token's decimals, prioritising the stored value if
    /// inconsistent.
    function decimalsForToken(mapping(address => TOFUTokenDecimals) storage sTOFUTokenDecimals, address token)
        internal
        returns (TOFUOutcome, uint8)
    {
        (TOFUOutcome tofuOutcome, uint8 readDecimals) = decimalsForTokenReadOnly(sTOFUTokenDecimals, token);

        if (tofuOutcome == TOFUOutcome.Initial) {
            sTOFUTokenDecimals[token] = TOFUTokenDecimals({initialized: true, tokenDecimals: readDecimals});
        }
        return (tofuOutcome, readDecimals);
    }
}
