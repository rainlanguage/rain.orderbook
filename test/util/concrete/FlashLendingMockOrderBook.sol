// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    TakeOrderConfigV4,
    IOV2,
    OrderV4,
    SignedContextV1,
    IOrderBookV5,
    TakeOrdersConfigV4,
    OrderConfigV4,
    ClearConfigV2,
    EvaluableV4,
    TaskV2,
    QuoteV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

contract FlashLendingMockOrderBook is IOrderBookV5 {
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        returns (bool)
    {
        receiver.onFlashLoan(msg.sender, token, amount, 0, data);
        return true;
    }

    function entask(TaskV2[] calldata) external pure {}

    /// @inheritdoc IOrderBookV5
    function quote2(QuoteV2 calldata) external pure returns (bool, uint256, uint256) {
        revert("quote");
    }

    /// @inheritdoc IOrderBookV5
    function takeOrders3(TakeOrdersConfigV4 calldata) external pure returns (uint256, uint256) {
        return (0, 0);
    }

    /// @inheritdoc IOrderBookV5
    function addOrder3(OrderConfigV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        return false;
    }

    function orderExists(bytes32) external pure returns (bool) {
        return false;
    }

    /// @inheritdoc IOrderBookV5
    function clear3(
        OrderV4 memory,
        OrderV4 memory,
        ClearConfigV2 calldata,
        SignedContextV1[] memory,
        SignedContextV1[] memory
    ) external {}
    function deposit3(address, uint256, uint256, TaskV2[] calldata) external {}
    function flashFee(address, uint256) external view returns (uint256) {}
    function maxFlashLoan(address) external view returns (uint256) {}
    function removeOrder3(OrderV4 calldata, TaskV2[] calldata) external returns (bool) {}

    function vaultBalance(address, address, uint256) external view returns (uint256) {}
    function withdraw3(address, uint256, uint256, TaskV2[] calldata) external {}
}
