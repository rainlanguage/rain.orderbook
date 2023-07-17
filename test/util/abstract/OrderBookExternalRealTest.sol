// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Test.sol";
import "rain.interpreter/concrete/RainterpreterNP.sol";
import "rain.interpreter/concrete/RainterpreterStore.sol";
import "rain.interpreter/concrete/RainterpreterExpressionDeployerNP.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibOrderBookConstants.sol";
import "test/util/abstract/IOrderBookV3Stub.sol";

import "src/concrete/OrderBook.sol";

abstract contract OrderBookExternalRealTest is Test, IOrderBookV3Stub {
    IInterpreterV1 immutable iInterpreter;
    IInterpreterStoreV1 immutable iStore;
    IExpressionDeployerV1 immutable iDeployer;
    IOrderBookV3 immutable iOrderbook;
    IERC20 immutable iToken0;
    IERC20 immutable iToken1;

    constructor() {
        vm.pauseGasMetering();
        iInterpreter = IInterpreterV1(new RainterpreterNP());
        iStore = IInterpreterStoreV1(new RainterpreterStore());

        // Deploy the expression deployer.
        bytes memory deployerMeta = LibRainterpreterExpressionDeployerNPMeta.authoringMeta();
        iDeployer = IExpressionDeployerV1(
            address(
                new RainterpreterExpressionDeployerNP(RainterpreterExpressionDeployerConstructionConfig(
                address(iInterpreter),
                address(iStore),
                deployerMeta
                ))
            )
        );
        // All non-mocked calls will revert.
        vm.etch(address(iDeployer), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(iInterpreter, iStore, address(0))
        );
        bytes memory orderbookMeta = vm.readFileBinary(ORDER_BOOK_META_PATH);
        console2.log("orderbook meta hash:");
        console2.logBytes(abi.encodePacked(keccak256(orderbookMeta)));
        iOrderbook = IOrderBookV3(
            address(new OrderBook(DeployerDiscoverableMetaV1ConstructionConfig(address(iDeployer), orderbookMeta)))
        );

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.resumeGasMetering();
    }
}
