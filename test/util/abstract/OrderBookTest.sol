// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";
import "rain.interpreter/interface/IExpressionDeployerV1.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibOrderBookConstants.sol";

import "src/interface/IOrderBookV2.sol";
import "src/concrete/OrderBook.sol";

/// @title OrderBookTest
/// Abstract contract that performs common setup needed for most orderbook tests.
///
/// Notably:
/// - Deploys a real orderbook contract with correct meta.
/// - Deploys several mockable token contracts.
/// - Deploys a mockable deployer contract for a DISpair.
///
/// Inherits from Test so that it can be used as a base contract for other tests.
abstract contract OrderBookTest is Test {
    IExpressionDeployerV1 immutable deployer;
    IOrderBookV2 immutable orderbook;
    IERC20 immutable token0;
    IERC20 immutable token1;

    constructor() {
        deployer = IExpressionDeployerV1(address(uint160(uint256(keccak256("deployer.rain.test")))));
        // All non-mocked calls will revert.
        vm.etch(address(deployer), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(deployer),
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(address(0), address(0), address(0))
        );
        bytes memory meta = vm.readFileBinary(ORDER_BOOK_META_PATH);
        console2.logBytes32(keccak256(meta));
        orderbook =
            IOrderBookV2(address(new OrderBook(DeployerDiscoverableMetaV1ConstructionConfig(address(deployer), meta))));

        token0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(token0), REVERTING_MOCK_BYTECODE);
        token1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(token1), REVERTING_MOCK_BYTECODE);
    }
}
