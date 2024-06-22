// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {IRouteProcessor} from "sushixswap-v2/src/interfaces/IRouteProcessor.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

import {
    OrderBookV3ArbOrderTaker,
    OrderBookV3ArbConfigV1,
    MinimumOutput
} from "../../abstract/OrderBookV3ArbOrderTaker.sol";

contract RouteProcessorOrderBookV3ArbOrderTaker is OrderBookV3ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    IRouteProcessor public immutable iRouteProcessor;

    constructor(OrderBookV3ArbConfigV1 memory config) OrderBookV3ArbOrderTaker(config) {
        (address routeProcessor) = abi.decode(config.implementationData, (address));
        iRouteProcessor = IRouteProcessor(routeProcessor);
    }

    /// @inheritdoc OrderBookV3ArbOrderTaker
    function onTakeOrders(
        address inputToken,
        address outputToken,
        uint256 inputAmountSent,
        uint256 totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        IERC20(inputToken).safeApprove(address(iRouteProcessor), 0);
        IERC20(inputToken).safeApprove(address(iRouteProcessor), type(uint256).max);
        bytes memory route = abi.decode(takeOrdersData, (bytes));
        (uint256 amountOut) = iRouteProcessor.processRoute(
            inputToken, inputAmountSent, outputToken, totalOutputAmount, address(this), route
        );
        IERC20(inputToken).safeApprove(address(iRouteProcessor), 0);
        (amountOut);
    }

    /// Allow receiving gas.
    fallback() external {}
}
