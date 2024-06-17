// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test, Vm, console2} from "forge-std/Test.sol";
import {RainterpreterNPE2} from "rain.interpreter/concrete/RainterpreterNPE2.sol";
import {RainterpreterStoreNPE2} from "rain.interpreter/concrete/RainterpreterStoreNPE2.sol";
import {
    RainterpreterExpressionDeployerNPE2,
    RainterpreterExpressionDeployerNPE2ConstructionConfigV2
} from "rain.interpreter/concrete/RainterpreterExpressionDeployerNPE2.sol";
import {LibAllStandardOpsNP} from "rain.interpreter/lib/op/LibAllStandardOpsNP.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IOrderBookV4Stub} from "test/util/abstract/IOrderBookV4Stub.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {IParserV2} from "rain.interpreter.interface/interface/unstable/IParserV2.sol";
import {IOrderBookV4, IInterpreterV3} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {OrderBook, IERC20} from "src/concrete/ob/OrderBook.sol";
import {IERC1820Registry} from "rain.erc1820/interface/IERC1820Registry.sol";
import {IERC1820_REGISTRY} from "rain.erc1820/lib/LibIERC1820.sol";
import {RainterpreterParserNPE2} from "rain.interpreter/concrete/RainterpreterParserNPE2.sol";

abstract contract OrderBookExternalRealTest is Test, IOrderBookV4Stub {
    IInterpreterV3 internal immutable iInterpreter;
    IInterpreterStoreV2 internal immutable iStore;
    IParserV2 internal immutable iParserV2;
    IOrderBookV4 internal immutable iOrderbook;
    IERC20 internal immutable iToken0;
    IERC20 internal immutable iToken1;

    constructor() {
        iInterpreter = IInterpreterV3(new RainterpreterNPE2());
        iStore = IInterpreterStoreV2(new RainterpreterStoreNPE2());
        address parser = address(new RainterpreterParserNPE2());
        iParserV2 = new RainterpreterExpressionDeployerNPE2(
            RainterpreterExpressionDeployerNPE2ConstructionConfigV2({
                interpreter: address(iInterpreter),
                store: address(iStore),
                parser: parser
            })
        );

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
        iOrderbook = IOrderBookV4(address(new OrderBook()));

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
    }
}
