// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "rain.interface.interpreter/IInterpreterStoreV1.sol";


contract RainterpreterStore is IInterpreterStoreV1 {
    
    function set(StateNamespace namespace_, uint256[] calldata kvs_) external {}

    function get(
        FullyQualifiedNamespace namespace_,
        uint256 key_
    ) external view returns (uint256) {}
}
