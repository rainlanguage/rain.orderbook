// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {
    OrderBookV4ArbOrderTaker,
    IOrderBookV4ArbOrderTaker,
    EvaluableV3,
    OrderBookV4ArbConfigV1,
    IOrderBookV4OrderTaker
} from "src/abstract/OrderBookV4ArbOrderTaker.sol";
import {IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV4ArbOrderTaker is OrderBookV4ArbOrderTaker {
    constructor()
        OrderBookV4ArbOrderTaker(
            OrderBookV4ArbConfigV1(
                address(0),
                EvaluableV3(IInterpreterV3(address(0)), IInterpreterStoreV2(address(0)), ""),
                abi.encode(address(0))
            )
        )
    {}
}

contract OrderBookV4ArbOrderTakerIERC165Test is Test {
    /// Test that ERC165 and IOrderBookV4ArbOrderTaker are supported interfaces
    /// as per ERC165.
    function testOrderBookV4ArbOrderTakerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV4ArbOrderTaker).interfaceId);
        vm.assume(badInterfaceId != type(IOrderBookV4OrderTaker).interfaceId);

        ChildOrderBookV4ArbOrderTaker arbOrderTaker = new ChildOrderBookV4ArbOrderTaker();
        assertTrue(arbOrderTaker.supportsInterface(type(IERC165).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV4ArbOrderTaker).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IOrderBookV4OrderTaker).interfaceId));
        assertFalse(arbOrderTaker.supportsInterface(badInterfaceId));
    }
}
