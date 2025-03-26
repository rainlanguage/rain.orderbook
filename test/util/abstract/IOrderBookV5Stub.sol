// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    IOrderBookV5,
    OrderConfigV4,
    OrderV4,
    ClearConfigV2,
    SignedContextV1,
    TakeOrdersConfigV4,
    EvaluableV4,
    TaskV2,
    QuoteV2,
    Float
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC3156FlashLender} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashLender.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

abstract contract IOrderBookV5Stub is IOrderBookV5 {
    /// @inheritdoc IOrderBookV5
    function entask2(TaskV2[] calldata) external pure {
        revert("eval");
    }

    /// @inheritdoc IOrderBookV5
    function quote(QuoteV2 calldata) external pure returns (bool, Float calldata, Float calldata) {
        revert("quote");
    }

    /// @inheritdoc IOrderBookV5
    function addOrder3(OrderConfigV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        revert("addOrder");
    }

    /// @inheritdoc IOrderBookV5
    function orderExists(bytes32) external pure returns (bool) {
        revert("orderExists");
    }

    /// @inheritdoc IOrderBookV5
    function removeOrder2(OrderV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        revert("removeOrder");
    }

    /// @inheritdoc IOrderBookV5
    function clear3(
        OrderV4 memory,
        OrderV4 memory,
        ClearConfigV2 calldata,
        SignedContextV1[] memory,
        SignedContextV1[] memory
    ) external pure {
        revert("clear");
    }

    /// @inheritdoc IOrderBookV5
    function deposit3(address, Float calldata, Float calldata, TaskV2[] calldata) external pure {
        revert("deposit");
    }

    /// @inheritdoc IOrderBookV5
    function takeOrders3(TakeOrdersConfigV4 calldata) external pure returns (Float calldata, Float calldata) {
        revert("takeOrders");
    }

    /// @inheritdoc IOrderBookV5
    function vaultBalance2(address, address, bytes32) external pure returns (Float calldata) {
        revert("vaultBalance");
    }

    /// @inheritdoc IOrderBookV5
    function withdraw3(address, bytes32, Float calldata, Float calldata, TaskV2[] calldata) external pure {
        revert("withdraw");
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
}
