// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

import {IOrderBookV3, OrderConfigV2, OrderV2} from "../IOrderBookV3.sol";
import {IExpressionDeployerV3} from "rain.interpreter/lib/caller/LibEvaluable.sol";
import {
    SignedContextV1
} from "rain.interpreter/interface/IInterpreterCallerV2.sol";

struct EvaluableConfigV4 {
    IExpressionDeployerV3 deployer;
    bytes bytecode;
    uint256[] constants;
    bytes meta;
}

struct EventContextV1 {
    uint256[] senderContext;
    SignedContextV1[] signedContexts;
}

enum EventsV1 {
    DEPOSIT,
    WITHDRAW,
    ADD_ORDER,
    REMOVE_ORDER,
    TOUCH
}

interface IOrderBookV4 is IOrderBookV3 {
    function on(EventsV1 evt, EvaluableConfigV4 calldata evaluableConfig) external;
    function off(EventsV1 evt, bytes32 evaluableHash) external;

    function touch(EventContextV1 calldata eventContext) external;

    function deposit(address token, uint256 vaultId, uint256 amount, EventContextV1 calldata eventContext) external;
    function withdraw(address token, uint256 vaultId, uint256 targetAmount, EventContextV1 calldata eventContext) external;

    function addOrder(OrderConfigV2 calldata config, EventContextV1 calldata eventContext) external returns (bool stateChanged);
    function removeOrder(OrderV2 calldata order, EventContextV1 calldata eventContext) external returns (bool stateChanged);
}