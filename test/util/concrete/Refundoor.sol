// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import "openzeppelin-contracts/contracts/utils/Address.sol";

contract Refundoor {
    fallback() external {
        Address.sendValue(payable(msg.sender), address(this).balance);
    }
}
