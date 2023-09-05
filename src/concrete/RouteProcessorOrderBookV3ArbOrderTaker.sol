// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "sushixswap-v2/src/interfaces/IRouteProcessor.sol";

import "../abstract/OrderBookV3ArbOrderTaker.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

bytes32 constant CALLER_META_HASH = bytes32(0xe45ec12419e88f2fed9938b77267520926dc4a2b09229e92f25c2f512ac445b2);

contract RouteProcessorOrderBookV3ArbOrderTaker is OrderBookV3ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    IRouteProcessor public sRouteProcessor;

    constructor(DeployerDiscoverableMetaV2ConstructionConfig memory config)
        OrderBookV3ArbOrderTaker(CALLER_META_HASH, config)
    {}

    /// @inheritdoc OrderBookV3ArbOrderTaker
    function _beforeInitialize(bytes memory data) internal virtual override {
        (address routeProcessor) = abi.decode(data, (address));
        sRouteProcessor = IRouteProcessor(routeProcessor);
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
        IERC20(inputToken).safeApprove(address(sRouteProcessor), 0);
        IERC20(inputToken).safeApprove(address(sRouteProcessor), type(uint256).max);
        bytes memory route = abi.decode(takeOrdersData, (bytes));
        (uint256 amountOut) = sRouteProcessor.processRoute(
            inputToken, inputAmountSent, outputToken, totalOutputAmount, address(this), route
        );
        IERC20(inputToken).safeApprove(address(sRouteProcessor), 0);
        (amountOut);
    }

    /// Allow receiving gas.
    fallback() external onlyNotInitializing {}
}
