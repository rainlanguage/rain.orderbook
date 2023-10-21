// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/forge-std/src/Test.sol";
import {RainterpreterNP} from "lib/rain.interpreter/src/concrete/RainterpreterNP.sol";
import "lib/rain.interpreter/src/concrete/RainterpreterStore.sol";
import "lib/rain.interpreter/src/concrete/RainterpreterExpressionDeployerNP.sol";
import "test/util/lib/LibTestConstants.sol";
import "test/util/abstract/IOrderBookV3Stub.sol";

import {OrderBook, IERC20, DeployerDiscoverableMetaV2ConstructionConfig} from "src/concrete/OrderBook.sol";

abstract contract OrderBookExternalRealTest is Test, IOrderBookV3Stub {
    IInterpreterV1 internal immutable iInterpreter;
    IInterpreterStoreV1 internal immutable iStore;
    IExpressionDeployerV2 internal immutable iDeployer;
    IOrderBookV3 internal immutable iOrderbook;
    IERC20 internal immutable iToken0;
    IERC20 internal immutable iToken1;

    constructor() {
        iInterpreter = IInterpreterV1(new RainterpreterNP());
        iStore = IInterpreterStoreV1(new RainterpreterStore());

        // Deploy the expression deployer.
        vm.etch(address(IERC1820_REGISTRY), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(IERC1820_REGISTRY),
            abi.encodeWithSelector(IERC1820Registry.interfaceHash.selector),
            abi.encode(bytes32(uint256(5)))
        );
        vm.mockCall(
            address(IERC1820_REGISTRY), abi.encodeWithSelector(IERC1820Registry.setInterfaceImplementer.selector), ""
        );
        bytes memory deployerMeta = LibAllStandardOpsNP.authoringMeta();
        console2.log("current deployer meta hash:");
        console2.logBytes32(keccak256(deployerMeta));
        iDeployer = IExpressionDeployerV2(
            address(
                new RainterpreterExpressionDeployerNP(RainterpreterExpressionDeployerConstructionConfig(
                address(iInterpreter),
                address(iStore),
                deployerMeta
                ))
            )
        );
        bytes memory orderbookMeta = vm.readFileBinary(ORDER_BOOK_META_PATH);
        console2.log("orderbook meta hash:");
        console2.logBytes(abi.encodePacked(keccak256(orderbookMeta)));
        iOrderbook = IOrderBookV3(
            address(new OrderBook(DeployerDiscoverableMetaV2ConstructionConfig(address(iDeployer), orderbookMeta)))
        );

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
    }
}
