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
/// Implements IOrderBookV2 so that it has access to all the relevant events.
abstract contract OrderBookTest is Test, IOrderBookV2 {
    IExpressionDeployerV1 immutable deployer;
    IOrderBookV2 immutable orderbook;
    IERC20 immutable token0;
    IERC20 immutable token1;

    constructor() {
        vm.pauseGasMetering();
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
        vm.resumeGasMetering();
    }

    /// @inheritdoc IOrderBookV2
    function addOrder(OrderConfig calldata) external pure {
        revert("addOrder");
    }

    /// @inheritdoc IOrderBookV2
    function clear(Order memory, Order memory, ClearConfig calldata, SignedContextV1[] memory, SignedContextV1[] memory)
        external
        pure
    {
        revert("clear");
    }

    /// @inheritdoc IOrderBookV2
    function deposit(DepositConfig calldata) external pure {
        revert("deposit");
    }

    /// @inheritdoc IERC3156FlashLender
    function flashLoan(IERC3156FlashBorrower, address, uint256, bytes calldata) external pure returns (bool) {
        revert("flashLoan");
    }

    /// @inheritdoc IERC3156FlashLender
    function flashFee(address, uint256) external pure returns (uint256) {
        revert("flashFee");
    }

    /// @inheritdoc IERC3156FlashLender
    function maxFlashLoan(address) external pure returns (uint256) {
        revert("maxFlashLoan");
    }

    /// @inheritdoc IOrderBookV2
    function removeOrder(Order calldata) external pure {
        revert("removeOrder");
    }

    /// @inheritdoc IOrderBookV2
    function takeOrders(TakeOrdersConfig calldata) external pure returns (uint256, uint256) {
        revert("takeOrders");
    }

    /// @inheritdoc IOrderBookV2
    function vaultBalance(address, address, uint256) external pure returns (uint256) {
        revert("vaultBalance");
    }

    /// @inheritdoc IOrderBookV2
    function withdraw(WithdrawConfig calldata) external pure {
        revert("withdraw");
    }
}
