// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract TestERC20 is ERC20 {
    uint8 private _decimals;

    constructor(string memory name_, string memory symbol_, uint8 decimals_, address recipient_, uint256 supply_)
        ERC20(name_, symbol_)
    {
        _decimals = decimals_;
        _mint(recipient_, supply_);
    }

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }
}
