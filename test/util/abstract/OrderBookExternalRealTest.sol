// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test, Vm, console2} from "forge-std/Test.sol";
import {RainterpreterNPE2} from "rain.interpreter/src/concrete/RainterpreterNPE2.sol";
import {RainterpreterStoreNPE2} from "rain.interpreter/src/concrete/RainterpreterStoreNPE2.sol";
import {
    RainterpreterExpressionDeployerNPE2,
    RainterpreterExpressionDeployerNPE2ConstructionConfig,
    CONSTRUCTION_META_HASH as DEPLOYER_CALLER_META_HASH
} from "rain.interpreter/src/concrete/RainterpreterExpressionDeployerNPE2.sol";
import {LibAllStandardOpsNP} from "rain.interpreter/src/lib/op/LibAllStandardOpsNP.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IOrderBookV3Stub} from "test/util/abstract/IOrderBookV3Stub.sol";
import {IInterpreterV2} from "rain.interpreter/src/interface/unstable/IInterpreterV2.sol";
import {IInterpreterStoreV2} from "rain.interpreter/src/interface/unstable/IInterpreterStoreV2.sol";
import {IExpressionDeployerV3} from "rain.interpreter/src/interface/unstable/IExpressionDeployerV3.sol";
import {IOrderBookV3} from "src/interface/unstable/IOrderBookV3.sol";
import {OrderBook, IERC20} from "src/concrete/OrderBook.sol";
import {IERC1820Registry} from "rain.erc1820/interface/IERC1820Registry.sol";
import {IERC1820_REGISTRY} from "rain.erc1820/lib/LibIERC1820.sol";
import {IParserV1} from "rain.interpreter/src/interface/IParserV1.sol";
import {RainterpreterParserNPE2} from "rain.interpreter/src/concrete/RainterpreterParserNPE2.sol";

string constant DEPLOYER_META_PATH = "lib/rain.interpreter/meta/RainterpreterExpressionDeployerNPE2.rain.meta";

abstract contract OrderBookExternalRealTest is Test, IOrderBookV3Stub {
    IExpressionDeployerV3 internal immutable iDeployer;
    IInterpreterV2 internal immutable iInterpreter;
    IInterpreterStoreV2 internal immutable iStore;
    IParserV1 internal immutable iParser;
    IOrderBookV3 internal immutable iOrderbook;
    IERC20 internal immutable iToken0;
    IERC20 internal immutable iToken1;

    constructor() {
        iInterpreter = IInterpreterV2(new RainterpreterNPE2());
        iStore = IInterpreterStoreV2(new RainterpreterStoreNPE2());
        iParser = IParserV1(new RainterpreterParserNPE2());

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
        bytes memory deployerMeta = vm.readFileBinary(DEPLOYER_META_PATH);
        bytes32 deployerMetaHash = keccak256(deployerMeta);
        if (deployerMetaHash != DEPLOYER_CALLER_META_HASH) {
            console2.log("deployer meta hash:");
            console2.logBytes32(deployerMetaHash);
            console2.log("expected deployer meta hash:");
            console2.logBytes32(DEPLOYER_CALLER_META_HASH);
        }
        iDeployer = IExpressionDeployerV3(
            address(
                new RainterpreterExpressionDeployerNPE2(
                    RainterpreterExpressionDeployerNPE2ConstructionConfig(
                        address(iInterpreter), address(iStore), address(iParser), deployerMeta
                    )
                )
            )
        );
        iOrderbook = IOrderBookV3(address(new OrderBook(address(iDeployer))));

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
    }
}
