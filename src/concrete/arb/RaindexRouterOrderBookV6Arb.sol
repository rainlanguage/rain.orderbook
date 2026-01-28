// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IRouteProcessor} from "sushixswap-v2/src/interfaces/IRouteProcessor.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {IOrderBookV6, Float} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";
import {
    OrderBookV6RaindexRouter,
    SafeERC20,
    IERC20,
    Address,
    TakeOrdersConfigV5,
    OrderBookV6ArbConfig
} from "../../abstract/OrderBookV6RaindexRouter.sol";

// Define possible route leg types
enum RouteLegType {
    RAINDEX,
    SUSHI,
    BALANCER,
    STABULL
}

// data and destination address of a route leg
// the data field needs to be decoded based on the type
struct RouteLeg {
    RouteLegType routeLegType;
    address destination;
    bytes data;
}

contract RaindexRouterOrderBookV6Arb is OrderBookV6RaindexRouter {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(OrderBookV6ArbConfig memory config) OrderBookV6RaindexRouter(config) {}

    /// @inheritdoc OrderBookV6RaindexRouter
    function _exchange(TakeOrdersConfigV5[] memory takeOrders, bytes memory exchangeData) internal virtual override {

        address prevLegTokenAddress = takeOrders[0].orders[0].order.validOutputs[takeOrders[0].orders[0].outputIOIndex].token;
        (Float startLegTotalOutput, Float startLegTotalInput) = IOrderBookV6(msg.sender).takeOrders4(takeOrders[0]);
        (startLegTotalInput);

        Float prevLegOutputAmount = startLegTotalOutput;

        if (exchangeData.length > 0) {
            RouteLeg[] memory routeLegs = abi.decode(exchangeData, (RouteLeg[]));

            for (uint256 i = 0; i < routeLegs.length; i++) {
                RouteLeg memory leg = routeLegs[i];

                if (leg.routeLegType == RouteLegType.SUSHI) {
                    (prevLegOutputAmount, prevLegTokenAddress) = _processSushiLeg(leg, prevLegOutputAmount, prevLegTokenAddress);
                } else if (leg.routeLegType == RouteLegType.RAINDEX) {
                    revert("raindex route leg type is not yet implemented");
                } else if (leg.routeLegType == RouteLegType.BALANCER) {
                    revert("balancer route leg type is not yet implemented");
                } else if (leg.routeLegType == RouteLegType.STABULL) {
                    revert("stabull route leg type is not yet implemented");
                }
            }
        }

        address endTakeOrdersInputToken = takeOrders[1].orders[0].order.validInputs[takeOrders[1].orders[0].inputIOIndex].token;
        IERC20(endTakeOrdersInputToken).forceApprove(msg.sender, 0);
        IERC20(endTakeOrdersInputToken).forceApprove(msg.sender, type(uint256).max);

        // set max io to previous leg output amount
        if (LibDecimalFloat.gt(takeOrders[1].maximumIO, prevLegOutputAmount)) {
            takeOrders[1].maximumIO = prevLegOutputAmount;
        }
        takeOrders[1].IOIsInput = false; // must always be false

        (Float finalLegTotalOutput, Float finalLegTotalInput) = IOrderBookV6(msg.sender).takeOrders4(takeOrders[1]);
        (finalLegTotalOutput, finalLegTotalInput);

        IERC20(endTakeOrdersInputToken).forceApprove(msg.sender, 0);
    }

    //slither-disable-next-line no-unused-vars
    function _processSushiLeg(
        RouteLeg memory routeLeg,
        Float prevLegOutputAmount,
        address prevLegTokenAddress
    ) internal returns (Float, address) {
        (address fromToken, address toToken, bytes memory route) = abi.decode(routeLeg.data, (address, address, bytes));
        
        require(prevLegTokenAddress == fromToken, "token mismatch");

        (uint256 fromTokenAmount, bool losslessInputAmount) =
            LibDecimalFloat.toFixedDecimalLossy(prevLegOutputAmount, IERC20Metadata(fromToken).decimals());
        (losslessInputAmount);

        uint8 toTokenDecimals = IERC20Metadata(toToken).decimals();
        (uint256 toTokenAmount, bool lossless) = LibDecimalFloat.toFixedDecimalLossy(LibDecimalFloat.FLOAT_ZERO, toTokenDecimals);
        if (!lossless) {
            toTokenAmount++;
        }

        IERC20(fromToken).forceApprove(routeLeg.destination, 0);
        IERC20(fromToken).forceApprove(routeLeg.destination, type(uint256).max);
        uint256 amountOut = IRouteProcessor(routeLeg.destination).processRoute(
            fromToken, fromTokenAmount, toToken, toTokenAmount, address(this), route
        );
        IERC20(fromToken).forceApprove(address(routeLeg.destination), 0);

        Float amountOutFloat = LibDecimalFloat.fromFixedDecimalLosslessPacked(amountOut, toTokenDecimals);

        return (amountOutFloat, toToken);
    }

    /// Allow receiving gas.
    fallback() external {}

    function a(RouteLeg calldata x) external {}
}
