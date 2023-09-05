// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "../abstract/OrderBookV3ArbOrderTaker.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

bytes32 constant CALLER_META_HASH = bytes32(0x9059bb5e174f68880ee6c97e424fb152304bfff2949242eca6ca1783e843fa32);

contract GenericPoolOrderBookV3ArbOrderTaker is OrderBookV3ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    constructor(DeployerDiscoverableMetaV2ConstructionConfig memory config)
        OrderBookV3ArbOrderTaker(CALLER_META_HASH, config)
    {}

    /// @inheritdoc OrderBookV3ArbOrderTaker
    function onTakeOrders(
        address inputToken,
        address outputToken,
        uint256 inputAmountSent,
        uint256 totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        (address spender, address pool, bytes memory encodedFunctionCall) =
            abi.decode(takeOrdersData, (address, address, bytes));

        IERC20(inputToken).safeApprove(spender, 0);
        IERC20(inputToken).safeApprove(spender, type(uint256).max);
        bytes memory returnData = pool.functionCallWithValue(encodedFunctionCall, address(this).balance);
        // Nothing can be done with returnData as `takeOrders` does not support
        // it.
        (returnData);
        IERC20(inputToken).safeApprove(spender, 0);
    }

    /// Allow receiving gas.
    fallback() external onlyNotInitializing {}
}
