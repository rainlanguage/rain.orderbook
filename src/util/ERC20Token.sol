// SPDX-License-Identifier: CAL
pragma solidity =0.8.19; 

import {ERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract ERC20Token is ERC20 { 

    constructor(string memory name_, string memory symbol_) ERC20(name_,symbol_) public {}

    function mint(address account, uint256 amount) external {
        _mint(account , amount) ; 
    }    

}