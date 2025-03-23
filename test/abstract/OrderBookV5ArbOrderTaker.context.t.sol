// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    ChildOrderBookV5ArbOrderTaker,
    TaskV2,
    SignedContextV1,
    EvaluableV4
} from "../util/concrete/ChildOrderBookV5ArbOrderTaker.sol";
import {OrderBookExternalRealTest} from "../util/abstract/OrderBookExternalRealTest.sol";
import {
    TakeOrdersConfigV3,
    TakeOrderConfigV3,
    IO,
    OrderConfigV3,
    OrderV3,
    IInterpreterV3
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {StateNamespace, LibNamespace} from "src/concrete/ob/OrderBook.sol";

contract OrderBookV5ArbOrderTakerContextTest is OrderBookExternalRealTest {
    function testOrderBookV5ArbOrderTakerContext() external {
        address alice = address(999999);
        address bob = address(999998);
        ChildOrderBookV5ArbOrderTaker arbOrderTaker = new ChildOrderBookV5ArbOrderTaker();

        OrderConfigV3 memory aliceOrderConfig;
        {
            IO[] memory aliceValidInputs = new IO[](1);
            aliceValidInputs[0] = IO({token: address(iToken0), decimals: 12, vaultId: 0});

            IO[] memory aliceValidOutputs = new IO[](1);
            aliceValidOutputs[0] = IO({token: address(iToken1), decimals: 12, vaultId: 0});

            aliceOrderConfig = OrderConfigV3({
                evaluable: EvaluableV4(iInterpreter, iStore, ""),
                validInputs: aliceValidInputs,
                validOutputs: aliceValidOutputs,
                nonce: 0,
                secret: 0,
                meta: ""
            });
        }

        OrderV3 memory aliceOrder = OrderV3({
            owner: alice,
            evaluable: aliceOrderConfig.evaluable,
            validInputs: aliceOrderConfig.validInputs,
            validOutputs: aliceOrderConfig.validOutputs,
            nonce: aliceOrderConfig.nonce
        });

        TakeOrderConfigV3 memory aliceTakeOrderConfig = TakeOrderConfigV3({
            order: aliceOrder,
            inputIOIndex: 0,
            outputIOIndex: 0,
            signedContext: new SignedContextV1[](0)
        });

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](1);
        orders[0] = aliceTakeOrderConfig;
        TakeOrdersConfigV3 memory takeOrdersConfig = TakeOrdersConfigV3({
            minimumInput: 0,
            maximumInput: type(uint256).max,
            maximumIORatio: type(uint256).max,
            orders: orders,
            data: ""
        });

        TaskV2 memory task = TaskV2({
            evaluable: EvaluableV4({
                interpreter: iInterpreter,
                store: iStore,
                bytecode: iParserV2.parse2(
                    bytes(
                        string.concat(
                            ":ensure(equal-to(context<1 0>() 3) \"input token\"),",
                            ":ensure(equal-to(context<1 1>() 4) \"output token\"),",
                            ":ensure(equal-to(context<1 2>() 5) \"gas balance\");"
                        )
                    )
                )
            }),
            signedContext: new SignedContextV1[](0)
        });

        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.approve.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.allowance.selector), abi.encode(0));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.balanceOf.selector, address(arbOrderTaker)),
            abi.encode(3e12)
        );
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.balanceOf.selector, address(arbOrderTaker)),
            abi.encode(4e12)
        );
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(12));
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(12));

        vm.deal(address(arbOrderTaker), 5e18);
        vm.prank(bob);
        arbOrderTaker.arb4(iOrderbook, takeOrdersConfig, task);
    }
}
