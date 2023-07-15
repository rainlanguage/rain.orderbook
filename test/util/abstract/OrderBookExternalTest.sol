// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import "forge-std/Test.sol";
import "rain.interpreter/interface/IExpressionDeployerV1.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibOrderBookConstants.sol";

import "src/interface/unstable/IOrderBookV3.sol";
import "src/concrete/OrderBook.sol";

/// @title OrderBookExternalTest
/// Abstract contract that performs common setup needed for testing an orderbook
/// from its external interface.
///
/// Notably:
/// - Deploys a real orderbook contract with correct meta.
/// - Deploys several mockable token contracts.
/// - Deploys a mockable deployer contract for a DISpair.
///
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Implements IOrderBookV2 so that it has access to all the relevant events.
abstract contract OrderBookExternalTest is Test, IOrderBookV3 {
    IInterpreterV1 immutable iInterpreter;
    IInterpreterStoreV1 immutable iStore;
    IExpressionDeployerV1 immutable iDeployer;
    IOrderBookV3 immutable iOrderbook;
    IERC20 immutable iToken0;
    IERC20 immutable iToken1;

    constructor() {
        vm.pauseGasMetering();
        iInterpreter = IInterpreterV1(address(uint160(uint256(keccak256("interpreter.rain.test")))));
        vm.etch(address(iInterpreter), REVERTING_MOCK_BYTECODE);
        iStore = IInterpreterStoreV1(address(uint160(uint256(keccak256("store.rain.test")))));
        vm.etch(address(iStore), REVERTING_MOCK_BYTECODE);
        iDeployer = IExpressionDeployerV1(address(uint160(uint256(keccak256("deployer.rain.test")))));
        // All non-mocked calls will revert.
        vm.etch(address(iDeployer), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(iInterpreter, iStore, address(0))
        );
        bytes memory meta = vm.readFileBinary(ORDER_BOOK_META_PATH);
        console2.log("meta hash:");
        console2.logBytes(abi.encodePacked(keccak256(meta)));
        iOrderbook =
            IOrderBookV3(address(new OrderBook(DeployerDiscoverableMetaV1ConstructionConfig(address(iDeployer), meta))));

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.resumeGasMetering();
    }

    /// @inheritdoc IOrderBookV3
    function addOrder(OrderConfig calldata) external pure {
        revert("addOrder");
    }

    /// @inheritdoc IOrderBookV3
    function clear(Order memory, Order memory, ClearConfig calldata, SignedContextV1[] memory, SignedContextV1[] memory)
        external
        pure
    {
        revert("clear");
    }

    /// @inheritdoc IOrderBookV3
    function deposit(address, uint256, uint256) external pure {
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

    /// @inheritdoc IOrderBookV3
    function removeOrder(Order calldata) external pure {
        revert("removeOrder");
    }

    /// @inheritdoc IOrderBookV3
    function takeOrders(TakeOrdersConfig calldata) external pure returns (uint256, uint256) {
        revert("takeOrders");
    }

    /// @inheritdoc IOrderBookV3
    function vaultBalance(address, address, uint256) external pure returns (uint256) {
        revert("vaultBalance");
    }

    /// @inheritdoc IOrderBookV3
    function withdraw(address, uint256, uint256) external pure {
        revert("withdraw");
    }
}
