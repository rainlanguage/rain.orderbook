// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {
    IOrderBookV3,
    OrderConfigV2,
    OrderV2,
    ClearConfig,
    SignedContextV1,
    IERC3156FlashBorrower,
    TakeOrdersConfigV2,
    IERC3156FlashLender
} from "rain.orderbook.interface/interface/IOrderBookV3.sol";

abstract contract IOrderBookV3Stub is IOrderBookV3 {
    /// @inheritdoc IOrderBookV3
    function addOrder(OrderConfigV2 calldata) external pure returns (bool) {
        revert("addOrder");
    }

    /// @inheritdoc IOrderBookV3
    function orderExists(bytes32) external pure returns (bool) {
        revert("orderExists");
    }

    /// @inheritdoc IOrderBookV3
    function removeOrder(OrderV2 calldata) external pure returns (bool) {
        revert("removeOrder");
    }

    /// @inheritdoc IOrderBookV3
    function clear(
        OrderV2 memory,
        OrderV2 memory,
        ClearConfig calldata,
        SignedContextV1[] memory,
        SignedContextV1[] memory
    ) external pure {
        revert("clear");
    }

    /// @inheritdoc IOrderBookV3
    function deposit(address, uint256, uint256) external pure {
        revert("deposit");
    }

    /// @inheritdoc IERC3156FlashLender
    function flashLoan(IERC3156FlashBorrower, address, uint256, bytes calldata) external pure returns (bool) {
        revert("flashLoan");
    }

    /// @inheritdoc IERC3156FlashLender
    function flashFee(address, uint256) external pure returns (uint256) {
        revert("flashFee");
    }

    /// @inheritdoc IERC3156FlashLender
    function maxFlashLoan(address) external pure returns (uint256) {
        revert("maxFlashLoan");
    }

    /// @inheritdoc IOrderBookV3
    function takeOrders(TakeOrdersConfigV2 calldata) external pure returns (uint256, uint256) {
        revert("takeOrders");
    }

    /// @inheritdoc IOrderBookV3
    function vaultBalance(address, address, uint256) external pure returns (uint256) {
        revert("vaultBalance");
    }

    /// @inheritdoc IOrderBookV3
    function withdraw(address, uint256, uint256) external pure {
        revert("withdraw");
    }
}
