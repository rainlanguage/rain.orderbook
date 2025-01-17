// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.18;

import {OrderV3} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

/// @title LibOrder
/// @notice Consistent handling of `OrderV3` for where it matters w.r.t.
/// determinism and security.
library LibOrder {
    /// Hashes `OrderV3` in a secure and deterministic way. Uses abi.encode
    /// rather than abi.encodePacked to guard against potential collisions where
    /// many inputs encode to the same output bytes.
    /// @param order The order to hash.
    /// @return The hash of `order`.
    function hash(OrderV3 memory order) internal pure returns (bytes32) {
        return keccak256(abi.encode(order));
    }
}
