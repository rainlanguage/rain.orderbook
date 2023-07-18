// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

import "../interface/unstable/IOrderBookV3.sol";

/// @title LibOrder
/// @notice Consistent handling of `Order` for where it matters w.r.t.
/// determinism and security.
library LibOrder {
    /// Hashes `Order` in a secure and deterministic way. Uses abi.encode rather
    /// than abi.encodePacked to guard against potential collisions where many
    /// inputs encode to the same output bytes.
    /// @param order The order to hash.
    /// @return The hash of `order`.
    function hash(Order memory order) internal pure returns (bytes32) {
        return keccak256(abi.encode(order));
    }
}
