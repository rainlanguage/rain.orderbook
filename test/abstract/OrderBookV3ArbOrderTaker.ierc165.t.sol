// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {
    OrderBookV3ArbOrderTaker,
    IOrderBookV3ArbOrderTaker,
    EvaluableConfigV3,
    OrderBookV3ArbConfigV1,
    IOrderBookV3OrderTaker
} from "src/abstract/OrderBookV3ArbOrderTaker.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/IExpressionDeployerV3.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV3ArbOrderTaker is OrderBookV3ArbOrderTaker {
    constructor()
        OrderBookV3ArbOrderTaker(
            OrderBookV3ArbConfigV1(
                address(0),
                EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
                abi.encode(address(0))
            )
        )
    {}
}

contract OrderBookV3ArbOrderTakerIERC165Test is Test {
    /// Test that ERC165 and IOrderBookV3ArbOrderTaker are supported interfaces
    /// as per ERC165.
    function testOrderBookV3ArbOrderTakerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV3ArbOrderTaker).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV3OrderTaker).interfaceId);

        ChildOrderBookV3ArbOrderTaker arbOrderTaker = new ChildOrderBookV3ArbOrderTaker();
        assertTrue(arbOrderTaker.supportsInterface(type(IERC165).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV3ArbOrderTaker).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV3OrderTaker).interfaceId));
        assertFalse(arbOrderTaker.supportsInterface(badInterfaceId));
    }
}
