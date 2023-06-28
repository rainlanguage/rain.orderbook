// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "src/interface/ierc3156/IERC3156FlashLender.sol";
import "src/interface/ierc3156/IERC3156FlashBorrower.sol";

import "src/abstract/OrderBookFlashBorrower.sol";

/// @dev Metadata hash for `DeployerDiscoverableMetaV1`.
/// - ABI for GenericPoolOrderBookFlashBorrower
/// - Interpreter caller metadata V1 for GenericPoolOrderBookFlashBorrower
bytes32 constant CALLER_META_HASH = bytes32(0x1dee96b6e546767de7f156f71db20a025bb92f75f4e58cd7f33d94dda4d7b81d);

/// @title GenericPoolOrderBookFlashBorrower
/// Implements the OrderBookFlashBorrower interface for a external liquidity
/// source that behaves vaguely like a standard AMM. The `exchangeData` from
/// `arb` is decoded into a spender, pool and callData. The `callData` is
/// literally the encoded function call to the pool. This allows the `arb`
/// caller to process a trade against any liquidity source that can swap tokens
/// within a single function call.
/// The `spender` is the address that will be approved to spend the input token
/// on `takeOrders`, which is almost always going to be the pool itself. If you
/// are unsure, simply set it to the pool address.
contract GenericPoolOrderBookFlashBorrower is OrderBookFlashBorrower {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(DeployerDiscoverableMetaV1ConstructionConfig memory config)
        OrderBookFlashBorrower(CALLER_META_HASH, config)
    {}

    /// @inheritdoc OrderBookFlashBorrower
    function _exchange(TakeOrdersConfig memory takeOrders, bytes memory exchangeData) internal virtual override {
        (address spender, address pool, bytes memory encodedFunctionCall) =
            abi.decode(exchangeData, (address, address, bytes));

        IERC20(takeOrders.input).safeApprove(spender, 0);
        IERC20(takeOrders.input).safeApprove(spender, type(uint256).max);
        bytes memory returnData = pool.functionCallWithValue(encodedFunctionCall, address(this).balance);
        // Nothing can be done with returnData as 3156 does not support it.
        (returnData);
        IERC20(takeOrders.input).safeApprove(spender, 0);
    }

    /// Allow receiving gas.
    fallback() external onlyNotInitializing {}
}
