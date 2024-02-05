// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

import {IOrderBookV3, OrderConfigV2, OrderV2} from "../IOrderBookV3.sol";
import {IExpressionDeployerV3} from "rain.interpreter/lib/caller/LibEvaluable.sol";
import {
    SignedContextV1
} from "rain.interpreter/interface/IInterpreterCallerV2.sol";

struct EvaluableV3 {
    IInterpreterV3 i9r;
    IInterpreterStoreV2 store;
    SourceIndexV2 sourceIndex;
    bytes bytecode;
    uint256[] constants;
    bytes meta;
}

struct OrderBookActionV1 {
    EvaluableV3 evaluable;
    uint256[] senderContext;
    SignedContextV1[] signedContexts;
}

interface IOrderBookV4 is IOrderBookV3 {
    function touch(OrderBookActionV1[] calldata actions) external;

    function deposit(address token, uint256 vaultId, uint256 amount, OrderBookActionV1[] calldata actions) external;
    function withdraw(address token, uint256 vaultId, uint256 targetAmount, OrderBookActionV1[] calldata actions) external;

    function addOrder(OrderConfigV2 calldata config, OrderBookActionV1[] calldata actions) external returns (bool stateChanged);
    function removeOrder(OrderV2 calldata order, OrderBookActionV1[] calldata actions) external returns (bool stateChanged);
}