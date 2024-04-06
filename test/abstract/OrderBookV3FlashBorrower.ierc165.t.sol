// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {
    OrderBookV3FlashBorrower,
    IERC3156FlashBorrower,
    EvaluableConfigV3,
    OrderBookV3ArbConfigV1
} from "src/abstract/OrderBookV3FlashBorrower.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/IExpressionDeployerV3.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV3FlashBorrower is OrderBookV3FlashBorrower {
    constructor()
        OrderBookV3FlashBorrower(
            OrderBookV3ArbConfigV1(
                address(0),
                EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
                abi.encode(address(0))
            )
        )
    {}
}

contract OrderBookV3FlashBorrowerIERC165Test is Test {
    /// Test that ERC165 and IERC3156FlashBorrower are supported interfaces
    /// as per ERC165.
    function testOrderBookV3FlashBorrowerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IERC3156FlashBorrower).interfaceId);

        ChildOrderBookV3FlashBorrower flashBorrower = new ChildOrderBookV3FlashBorrower();
        assertTrue(flashBorrower.supportsInterface(type(IERC165).interfaceId));
        assertTrue(flashBorrower.supportsInterface(type(IERC3156FlashBorrower).interfaceId));
        assertFalse(flashBorrower.supportsInterface(badInterfaceId));
    }
}
