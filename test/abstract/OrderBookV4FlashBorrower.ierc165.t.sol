// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {
    OrderBookV4FlashBorrower,
    IERC3156FlashBorrower,
    EvaluableV3,
    OrderBookV4ArbConfigV1
} from "src/abstract/OrderBookV4FlashBorrower.sol";
import {IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV4FlashBorrower is OrderBookV4FlashBorrower {
    constructor()
        OrderBookV4FlashBorrower(
            OrderBookV4ArbConfigV1(
                address(0),
                EvaluableV3(IInterpreterV3(address(0)), IInterpreterStoreV2(address(0)), ""),
                abi.encode(address(0))
            )
        )
    {}
}

contract OrderBookV4FlashBorrowerIERC165Test is Test {
    /// Test that ERC165 and IERC3156FlashBorrower are supported interfaces
    /// as per ERC165.
    function testOrderBookV4FlashBorrowerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IERC3156FlashBorrower).interfaceId);

        ChildOrderBookV4FlashBorrower flashBorrower = new ChildOrderBookV4FlashBorrower();
        assertTrue(flashBorrower.supportsInterface(type(IERC165).interfaceId));
        assertTrue(flashBorrower.supportsInterface(type(IERC3156FlashBorrower).interfaceId));
        assertFalse(flashBorrower.supportsInterface(badInterfaceId));
    }
}
