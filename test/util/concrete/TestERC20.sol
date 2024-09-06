// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract TestERC20 is ERC20 {
    constructor(string memory name, string memory symbol, address recipient, uint256 supply) ERC20(name, symbol) {
        _mint(recipient, supply);
    }
}