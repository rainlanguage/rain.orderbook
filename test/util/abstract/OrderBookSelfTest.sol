// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test} from "lib/forge-std/src/Test.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";

import {OrderBook, IExpressionDeployerV3} from "src/concrete/OrderBook.sol";

/// @title OrderBookSelfTest
/// Abstract contract that is an `OrderBook` and can be used to test itself.
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Mocks all externalities during construction.
abstract contract OrderBookSelfTest is Test, OrderBook {
    function constructDeployer() internal returns (address deployer) {
        vm.pauseGasMetering();
        deployer = address(uint160(uint256(keccak256("deployer.rain.test"))));
        // All non-mocked calls will revert.
        vm.etch(address(deployer), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(deployer),
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(address(0), address(0), address(0), "00020000")
        );
        vm.resumeGasMetering();
    }

    constructor() OrderBook(constructDeployer()) {}
}
