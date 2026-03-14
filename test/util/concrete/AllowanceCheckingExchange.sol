// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

/// @dev Exchange that records the allowance it sees from its caller at call time.
contract AllowanceCheckingExchange {
    uint256 public lastAllowance;
    uint256 public lastEthReceived;

    function swap(IERC20 tokenIn, IERC20 tokenOut, uint256 amountIn) external payable {
        lastAllowance = tokenIn.allowance(msg.sender, address(this));
        lastEthReceived = msg.value;
        tokenIn.transferFrom(msg.sender, address(this), amountIn);
        tokenOut.transfer(msg.sender, amountIn);
        if (msg.value > 0) {
            (bool sent,) = payable(msg.sender).call{value: msg.value}("");
            require(sent, "ETH return failed");
        }
    }
}
