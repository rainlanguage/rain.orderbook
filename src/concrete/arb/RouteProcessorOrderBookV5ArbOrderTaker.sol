// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IRouteProcessor} from "sushixswap-v2/src/interfaces/IRouteProcessor.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

import {OrderBookV5ArbOrderTaker, OrderBookV5ArbConfig, Float} from "../../abstract/OrderBookV5ArbOrderTaker.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract RouteProcessorOrderBookV5ArbOrderTaker is OrderBookV5ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    IRouteProcessor public immutable iRouteProcessor;

    constructor(OrderBookV5ArbConfig memory config) OrderBookV5ArbOrderTaker(config) {
        (address routeProcessor) = abi.decode(config.implementationData, (address));
        iRouteProcessor = IRouteProcessor(routeProcessor);
    }

    /// @inheritdoc OrderBookV5ArbOrderTaker
    function onTakeOrders2(
        address inputToken,
        address outputToken,
        Float calldata inputAmountSent,
        Float calldata totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders2(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        IERC20(inputToken).safeApprove(address(iRouteProcessor), 0);
        IERC20(inputToken).safeApprove(address(iRouteProcessor), type(uint256).max);
        bytes memory route = abi.decode(takeOrdersData, (bytes));
        (uint256 amountOut) = iRouteProcessor.processRoute(
            inputToken,
            LibDecimalFloat.toFixedDecimalLossless(
                inputAmountSent.signedCoefficient, inputAmountSent.exponent, IERC20Metadata(inputToken).decimals()
            ),
            outputToken,
            LibDecimalFloat.toFixedDecimalLossless(
                totalOutputAmount.signedCoefficient, totalOutputAmount.exponent, IERC20Metadata(outputToken).decimals()
            ),
            address(this),
            route
        );
        IERC20(inputToken).safeApprove(address(iRouteProcessor), 0);
        (amountOut);
    }

    /// Allow receiving gas.
    fallback() external {}
}
