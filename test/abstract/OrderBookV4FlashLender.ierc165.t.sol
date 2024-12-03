// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {OrderBookV4FlashLender, IERC3156FlashLender} from "src/abstract/OrderBookV4FlashLender.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV4FlashLender is OrderBookV4FlashLender {
    constructor() OrderBookV4FlashLender() {}
}

contract OrderBookV4FlashLenderIERC165Test is Test {
    /// Test that ERC165 and IERC3156FlashLender are supported interfaces
    /// as per ERC165.
    function testOrderBookV4FlashLenderIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IERC3156FlashLender).interfaceId);

        ChildOrderBookV4FlashLender flashLender = new ChildOrderBookV4FlashLender();
        assertTrue(flashLender.supportsInterface(type(IERC165).interfaceId));
        assertTrue(flashLender.supportsInterface(type(IERC3156FlashLender).interfaceId));
        assertFalse(flashLender.supportsInterface(badInterfaceId));
    }
}
