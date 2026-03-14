// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

import {
    GenericPoolOrderBookV6FlashBorrower,
    OrderBookV6ArbConfig
} from "src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    OrderV4,
    IOV2,
    EvaluableV4,
    SignedContextV1,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {AllowanceCheckingExchange} from "test/util/concrete/AllowanceCheckingExchange.sol";
import {RealisticFlashLendingMockOrderBook} from "test/util/concrete/RealisticFlashLendingMockOrderBook.sol";

/// After a successful arb4, the spender's allowance on the borrowed token
/// from the arb contract must be zero (approve-call-revoke).
contract GenericPoolOrderBookV6FlashBorrowerApprovalRevokedTest is Test {
    function testApprovalRevokedAfterExchange() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        RealisticFlashLendingMockOrderBook mockOb = new RealisticFlashLendingMockOrderBook();
        vm.etch(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, address(mockOb).code);
        RealisticFlashLendingMockOrderBook orderBook =
            RealisticFlashLendingMockOrderBook(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);

        AllowanceCheckingExchange exchange = new AllowanceCheckingExchange();

        outputToken.mint(address(orderBook), 1000e18);
        inputToken.mint(address(exchange), 100e18);

        GenericPoolOrderBookV6FlashBorrower arb = new GenericPoolOrderBookV6FlashBorrower(
            OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                }),
                ""
            )
        );

        IOV2[] memory validInputs = new IOV2[](1);
        validInputs[0] = IOV2(address(inputToken), bytes32(0));
        IOV2[] memory validOutputs = new IOV2[](1);
        validOutputs[0] = IOV2(address(outputToken), bytes32(0));

        OrderV4 memory order = OrderV4({
            owner: address(0x1234),
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: bytes32(0)
        });

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));

        bytes memory exchangeData = abi.encode(
            address(exchange),
            address(exchange),
            abi.encodeCall(
                AllowanceCheckingExchange.swap, (IERC20(address(outputToken)), IERC20(address(inputToken)), 100e18)
            )
        );

        arb.arb4(
            IRaindexV6(address(orderBook)),
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(100, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: ""
            }),
            exchangeData,
            TaskV2({
                evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                signedContext: new SignedContextV1[](0)
            })
        );

        // During the call the exchange saw max approval.
        assertEq(exchange.lastAllowance(), type(uint256).max);

        // After arb4 completes, the spender allowance is revoked to zero.
        assertEq(outputToken.allowance(address(arb), address(exchange)), 0);
    }
}
