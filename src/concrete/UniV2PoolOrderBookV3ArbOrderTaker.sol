// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol";

import "../abstract/OrderBookV3ArbOrderTaker.sol";
import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "lib/openzeppelin-contracts/contracts/utils/Address.sol";

bytes32 constant CALLER_META_HASH = bytes32(0x00);

contract UniV2PoolOrderBookV3ArbOrderTaker is OrderBookV3ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    IUniswapV2Router02 public sUniV2Router02;

    constructor(DeployerDiscoverableMetaV2ConstructionConfig memory config)
        OrderBookV3ArbOrderTaker(CALLER_META_HASH, config)
    {}

    /// @inheritdoc OrderBookV3ArbOrderTaker
    function _beforeInitialize(bytes memory data) internal virtual override {
        (address uniV2Router02) = abi.decode(data, (address));
        sUniV2Router02 = IUniswapV2Router02(uniV2Router02);
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
        IERC20(inputToken).safeApprove(address(sUniV2Router02), 0);
        IERC20(inputToken).safeApprove(address(sUniV2Router02), type(uint256).max);
        address[] memory path = new address[](2);
        path[0] = inputToken;
        path[1] = outputToken;
        (uint256[] memory amounts) = sUniV2Router02.swapExactTokensForTokens(
            inputAmountSent, totalOutputAmount, path, address(this), block.timestamp
        );
        IERC20(inputToken).safeApprove(address(sUniV2Router02), 0);
        (amounts);
    }

    /// Allow receiving gas.
    fallback() external onlyNotInitializing {}
}
