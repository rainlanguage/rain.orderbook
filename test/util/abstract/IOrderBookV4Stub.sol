// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {
    IOrderBookV4,
    OrderConfigV3,
    OrderV3,
    ClearConfig,
    SignedContextV1,
    TakeOrdersConfigV3,
    EvaluableV3,
    TaskV1,
    Quote
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IERC3156FlashLender} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashLender.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

abstract contract IOrderBookV4Stub is IOrderBookV4 {
    /// @inheritdoc IOrderBookV4
    function entask(TaskV1[] calldata) external pure {
        revert("eval");
    }

    /// @inheritdoc IOrderBookV4
    function quote(Quote calldata) external pure returns (bool, uint256, uint256) {
        revert("quote");
    }

    /// @inheritdoc IOrderBookV4
    function addOrder2(OrderConfigV3 calldata, TaskV1[] calldata) external pure returns (bool) {
        revert("addOrder");
    }

    /// @inheritdoc IOrderBookV4
    function orderExists(bytes32) external pure returns (bool) {
        revert("orderExists");
    }

    /// @inheritdoc IOrderBookV4
    function removeOrder2(OrderV3 calldata, TaskV1[] calldata) external pure returns (bool) {
        revert("removeOrder");
    }

    /// @inheritdoc IOrderBookV4
    function clear2(
        OrderV3 memory,
        OrderV3 memory,
        ClearConfig calldata,
        SignedContextV1[] memory,
        SignedContextV1[] memory
    ) external pure {
        revert("clear");
    }

    /// @inheritdoc IOrderBookV4
    function deposit2(address, uint256, uint256, TaskV1[] calldata) external pure {
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

    /// @inheritdoc IOrderBookV4
    function takeOrders2(TakeOrdersConfigV3 calldata) external pure returns (uint256, uint256) {
        revert("takeOrders");
    }

    /// @inheritdoc IOrderBookV4
    function vaultBalance(address, address, uint256) external pure returns (uint256) {
        revert("vaultBalance");
    }

    /// @inheritdoc IOrderBookV4
    function withdraw2(address, uint256, uint256, TaskV1[] calldata) external pure {
        revert("withdraw");
    }
}
