// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Test.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibOrderBookConstants.sol";

import "src/concrete/OrderBook.sol";

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
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(address(0), address(0), address(0))
        );
        vm.resumeGasMetering();
    }

    function constructMeta() internal returns (bytes memory meta) {
        vm.pauseGasMetering();
        meta = vm.readFileBinary(ORDER_BOOK_META_PATH);
        vm.resumeGasMetering();
    }

    constructor() OrderBook(DeployerDiscoverableMetaV1ConstructionConfig(constructDeployer(), constructMeta())) {}
}
