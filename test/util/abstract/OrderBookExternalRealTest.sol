// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, Vm, console2} from "forge-std/Test.sol";
import {Rainterpreter} from "rain.interpreter/concrete/Rainterpreter.sol";
import {RainterpreterStore} from "rain.interpreter/concrete/RainterpreterStore.sol";
import {
    RainterpreterExpressionDeployer,
    RainterpreterExpressionDeployerConstructionConfigV2
} from "rain.interpreter/concrete/RainterpreterExpressionDeployer.sol";
import {LibAllStandardOpsNP} from "rain.interpreter/lib/op/LibAllStandardOpsNP.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IOrderBookV4Stub} from "test/util/abstract/IOrderBookV4Stub.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {
    IOrderBookV4,
    IInterpreterV3,
    TaskV1,
    EvaluableV3,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {OrderBook, IERC20} from "src/concrete/ob/OrderBook.sol";
import {RainterpreterParser} from "rain.interpreter/concrete/RainterpreterParser.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

abstract contract OrderBookExternalRealTest is Test, IOrderBookV4Stub {
    IInterpreterV3 internal immutable iInterpreter;
    IInterpreterStoreV2 internal immutable iStore;
    RainterpreterParserNPE2 internal immutable iParser;
    IParserV2 internal immutable iParserV2;
    IOrderBookV4 internal immutable iOrderbook;
    IERC20 internal immutable iToken0;
    IERC20 internal immutable iToken1;
    OrderBookSubParser internal immutable iSubParser;

    constructor() {
        iInterpreter = IInterpreterV3(new RainterpreterNPE2());
        iStore = IInterpreterStoreV2(new RainterpreterStoreNPE2());
        iParser = new RainterpreterParserNPE2();
        iParserV2 = new RainterpreterExpressionDeployerNPE2(
            RainterpreterExpressionDeployerNPE2ConstructionConfigV2({
                interpreter: address(iInterpreter),
                store: address(iStore),
                parser: address(iParser)
            })
        );

        iOrderbook = IOrderBookV4(address(new OrderBook()));

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(18));

        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(18));

        iSubParser = new OrderBookSubParser();
    }

    function assumeEtchable(address account) internal view {
        assumeNotPrecompile(account);
        vm.assume(account != address(iInterpreter));
        vm.assume(account != address(iStore));
        vm.assume(account != address(iParserV2));
        vm.assume(account != address(iParser));

        vm.assume(account != address(iOrderbook));
        vm.assume(account != address(iToken0));
        vm.assume(account != address(iToken1));
        vm.assume(account != address(iSubParser));

        vm.assume(account != address(this));
        vm.assume(account != address(vm));
        // The console.
        vm.assume(account != address(0x000000000000000000636F6e736F6c652e6c6f67));
    }

    function evalsToActions(bytes[] memory evals) internal view returns (TaskV1[] memory) {
        TaskV1[] memory actions = new TaskV1[](evals.length);
        for (uint256 i = 0; i < evals.length; i++) {
            actions[i] = TaskV1(EvaluableV3(iInterpreter, iStore, iParserV2.parse2(evals[i])), new SignedContextV1[](0));
        }
        return actions;
    }
}
