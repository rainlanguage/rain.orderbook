// SPDX-License-Identifier: UNLICENSED

pragma solidity =0.8.19;

import {ERC20, ERC20Burnable} from "lib/openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Burnable.sol";


contract ERC20Test is ERC20Burnable{
    constructor() ERC20("TestToken", "TT") {}


    // Mint tokens for specific address
    function mint(uint256 amount, address owner) public virtual {
        _mint(owner, amount);
    }
}