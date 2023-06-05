// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "rain.interface.interpreter/IInterpreterStoreV1.sol";
import "rain.lib.interpreter/LibInterpreterState.sol";
import "rain.lib.interpreter/LibNamespace.sol";


contract RainterpreterStore is IInterpreterStoreV1 {
    using LibNamespace for StateNamespace;

    mapping(FullyQualifiedNamespace => mapping(uint256 => uint256))
        internal store;

    /// @inheritdoc IInterpreterStoreV1
    function set(StateNamespace namespace_, uint256[] calldata kvs_) external {
        unchecked {
            FullyQualifiedNamespace fullyQualifiedNamespace_ = namespace_
                .qualifyNamespace();
            for (uint256 i_ = 0; i_ < kvs_.length; i_ += 2) {
                store[fullyQualifiedNamespace_][kvs_[i_]] = kvs_[i_ + 1];
            }
        }
    }

    /// @inheritdoc IInterpreterStoreV1
    function get(
        FullyQualifiedNamespace namespace_,
        uint256 key_
    ) external view returns (uint256) {
        return store[namespace_][key_];
    }
}
