// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "openzeppelin-contracts/contracts/utils/Address.sol";

/// @title Reenteroor
/// A contract that reenters the caller with a configurable call.
contract Reenteroor {
    using Address for address;

    /// The call to reenter with. Set by `reenterWith`.
    bytes internal _sEncodedCall;

    /// Set the call to reenter with. The encoding will be used by the fallback
    /// to call back into the caller.
    function reenterWith(bytes memory encodedCall) external {
        _sEncodedCall = encodedCall;
    }

    /// Reenter the caller with the call set by `reenterWith`. This will bubble
    /// up any reverts from the reentrant call so tests can check that
    /// reentrancy guards are working.
    fallback() external {
        address(msg.sender).functionCall(_sEncodedCall);
    }
}
