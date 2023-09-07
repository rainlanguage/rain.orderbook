// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "openzeppelin-contracts/contracts/utils/Address.sol";

contract Refundoor {
    fallback() external {
        Address.sendValue(payable(msg.sender), address(this).balance);
    }
}
