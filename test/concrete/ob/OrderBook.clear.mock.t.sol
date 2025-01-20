// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, stdError} from "forge-std/Test.sol";

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {
    OrderConfigV3,
    OrderV3,
    IO,
    ClearConfig,
    EvaluableV3,
    SignedContextV1,
    IInterpreterV3,
    TaskV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {StateNamespace} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {LibFixedPointDecimalArithmeticOpenZeppelin} from
    "rain.math.fixedpoint/lib/LibFixedPointDecimalArithmeticOpenZeppelin.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
    using LibFixedPointDecimalArithmeticOpenZeppelin for uint256;
    using Math for uint256;

    /// Make a deposit to the OB mocking the internal transferFrom call.
    function _depositInternal(address depositor, address token, uint256 vaultId, uint256 amount) internal {
        vm.prank(depositor);
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(true)
        );
        iOrderbook.deposit2(address(token), vaultId, amount, new TaskV1[](0));

        assertEq(iOrderbook.vaultBalance(depositor, token, vaultId), amount);
    }

    function conformBasicConfig(OrderConfigV3 memory aliceConfig, OrderConfigV3 memory bobConfig) internal view {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);

        aliceConfig.evaluable.interpreter = iInterpreter;
        aliceConfig.evaluable.store = iStore;

        bobConfig.evaluable.interpreter = iInterpreter;
        bobConfig.evaluable.store = iStore;

        aliceConfig.validInputs[0].token = address(iToken0);
        aliceConfig.validOutputs[0].token = address(iToken1);

        bobConfig.validInputs[0].token = address(iToken1);
        bobConfig.validOutputs[0].token = address(iToken0);

        aliceConfig.validInputs[0].decimals = 18;
        aliceConfig.validOutputs[0].decimals = 18;
        bobConfig.validInputs[0].decimals = 18;
        bobConfig.validOutputs[0].decimals = 18;

        aliceConfig.meta = "";
        bobConfig.meta = "";
    }

    struct CheckClear {
        address alice;
        OrderConfigV3 aliceConfig;
        address bob;
        OrderConfigV3 bobConfig;
        address bountyBot;
        uint256 aliceBountyVaultId;
        uint256 bobBountyVaultId;
        uint256 aliceAmount;
        uint256 bobAmount;
        bytes expression;
        uint256[] orderStackAlice;
        uint256[] orderStackBob;
        uint256 expectedAliceOutput;
        uint256 expectedBobOutput;
        uint256 expectedAliceInput;
        uint256 expectedBobInput;
        bytes expectedError;
    }

    function checkClear(CheckClear memory clear) internal {
        vm.assume(clear.alice != clear.bob);
        vm.assume(clear.alice != clear.bountyBot);
        vm.assume(clear.bob != clear.bountyBot);
        vm.assume(clear.aliceBountyVaultId != clear.bobBountyVaultId);

        conformBasicConfig(clear.aliceConfig, clear.bobConfig);
        vm.assume(keccak256(clear.aliceConfig.evaluable.bytecode) != keccak256(clear.bobConfig.evaluable.bytecode));

        _depositInternal(
            clear.alice,
            clear.aliceConfig.validOutputs[0].token,
            clear.aliceConfig.validOutputs[0].vaultId,
            clear.aliceAmount
        );
        _depositInternal(
            clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId, clear.bobAmount
        );

        assertEq(
            iOrderbook.vaultBalance(
                clear.alice, clear.aliceConfig.validInputs[0].token, clear.aliceConfig.validInputs[0].vaultId
            ),
            0
        );
        assertEq(
            iOrderbook.vaultBalance(
                clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
            ),
            clear.aliceAmount
        );

        assertEq(
            iOrderbook.vaultBalance(
                clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId
            ),
            0
        );
        assertEq(
            iOrderbook.vaultBalance(
                clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
            ),
            clear.bobAmount
        );

        assertEq(iOrderbook.vaultBalance(clear.bountyBot, address(iToken0), clear.aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(clear.bountyBot, address(iToken1), clear.bobBountyVaultId), 0);

        {
            {
                bytes memory call = abi.encodeWithSelector(
                    IInterpreterV3.eval3.selector,
                    clear.aliceConfig.evaluable.store,
                    LibNamespace.qualifyNamespace(
                        StateNamespace.wrap(uint256(uint160(clear.alice))), address(iOrderbook)
                    )
                );

                vm.mockCall(address(iInterpreter), call, abi.encode(clear.orderStackAlice, new uint256[](0)));

                call = abi.encodeWithSelector(
                    IInterpreterV3.eval3.selector,
                    clear.bobConfig.evaluable.store,
                    LibNamespace.qualifyNamespace(StateNamespace.wrap(uint256(uint160(clear.bob))), address(iOrderbook))
                );

                vm.mockCall(address(iInterpreter), call, abi.encode(clear.orderStackBob, new uint256[](0)));
            }

            OrderV3 memory aliceOrder;
            OrderV3 memory bobOrder;
            {
                (aliceOrder,) = addOrderWithChecks(clear.alice, clear.aliceConfig, clear.expression);
                (bobOrder,) = addOrderWithChecks(clear.bob, clear.bobConfig, clear.expression);
            }

            ClearConfig memory configClear = ClearConfig({
                aliceInputIOIndex: 0,
                aliceOutputIOIndex: 0,
                bobInputIOIndex: 0,
                bobOutputIOIndex: 0,
                aliceBountyVaultId: clear.aliceBountyVaultId,
                bobBountyVaultId: clear.bobBountyVaultId
            });

            vm.prank(clear.bountyBot);
            if (clear.expectedError.length > 0) {
                vm.expectRevert(clear.expectedError);
            }
            iOrderbook.clear2(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
        }

        assertEq(
            iOrderbook.vaultBalance(
                clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
            ),
            clear.aliceAmount - clear.expectedAliceOutput,
            "Alice output vault"
        );
        assertEq(
            iOrderbook.vaultBalance(
                clear.alice, clear.aliceConfig.validInputs[0].token, clear.aliceConfig.validInputs[0].vaultId
            ),
            clear.expectedAliceInput,
            "Alice input vault"
        );

        assertEq(
            iOrderbook.vaultBalance(
                clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
            ),
            clear.bobAmount - clear.expectedBobOutput,
            "Bob output vault"
        );
        assertEq(
            iOrderbook.vaultBalance(
                clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId
            ),
            clear.expectedBobInput,
            // clear.expectedBobOutput.fixedPointMul(clear.orderStackBob[0], Math.Rounding.Up),
            "Bob input vault"
        );

        assertEq(
            iOrderbook.vaultBalance(clear.bountyBot, clear.aliceConfig.validOutputs[0].token, clear.aliceBountyVaultId),
            clear.expectedAliceOutput - clear.expectedBobInput,
            "Alice bounty"
        );
        assertEq(
            iOrderbook.vaultBalance(clear.bountyBot, clear.aliceConfig.validInputs[0].token, clear.aliceBountyVaultId),
            0,
            "Alice bounty input"
        );
        assertEq(
            iOrderbook.vaultBalance(clear.bountyBot, clear.bobConfig.validOutputs[0].token, clear.bobBountyVaultId),
            clear.expectedBobOutput - clear.expectedAliceInput,
            "Bob bounty"
        );
        assertEq(
            iOrderbook.vaultBalance(clear.bountyBot, clear.bobConfig.validInputs[0].token, clear.bobBountyVaultId),
            0,
            "Bob bounty input"
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearSimple(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) external {
        uint256 aliceAmount = 2e18;
        uint256 bobAmount = 2e18;

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        uint256[] memory orderStackAlice = new uint256[](2);
        orderStackAlice[0] = 0.99e18; // orderIORatio
        orderStackAlice[1] = 0.5e18; // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                aliceAmount,
                bobAmount,
                expression,
                orderStackAlice,
                orderStackAlice,
                0.5e18,
                0.5e18,
                0.495e18,
                0.495e18,
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatio(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId,
        uint256 aliceIORatio,
        uint256 bobIORatio
    ) external {
        // 0 tested separately.
        aliceIORatio = bound(aliceIORatio, 1, 1e18);
        bobIORatio = bound(bobIORatio, 1e18, uint256(1e18).fixedPointDiv(aliceIORatio, Math.Rounding.Down));

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        uint256[] memory orderStackAlice = new uint256[](2);
        orderStackAlice[0] = aliceIORatio; // orderIORatio
        orderStackAlice[1] = 1e18; // orderOutputMax

        uint256[] memory orderStackBob = new uint256[](2);
        orderStackBob[0] = bobIORatio; // orderIORatio
        orderStackBob[1] = 1e18; // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                1e18,
                1e18,
                expression,
                orderStackAlice,
                orderStackBob,
                1e18,
                // Alice is outputting 1 so bob will output enough to match this
                // according to his own IO ratio.
                uint256(1e18).fixedPointDiv(bobIORatio, Math.Rounding.Down).min(1e18),
                // Expected input for Alice is aliceOutput * aliceIORatio
                uint256(1e18).fixedPointMul(aliceIORatio, Math.Rounding.Up),
                // Expected input for Bob is Alice's output in entirety, because
                // alice IO * bob IO <= 1 and Bob is the larger ratio.
                // As Bob's ratio is >= 1 he will have his input shrunk to match
                // Alice's output. This means in this case Bob's input will be
                // 1 always, as it either = 1 anyway or matches Alice's 1.
                uint256(1e18),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatioError(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId,
        uint256 aliceIORatio,
        uint256 bobIORatio
    ) external {
        aliceIORatio = bound(aliceIORatio, 1e18 + 1, 1.1e18);
        bobIORatio = bound(bobIORatio, 1e18 + 1, 1.1e18);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        uint256[] memory orderStackAlice = new uint256[](2);
        orderStackAlice[0] = aliceIORatio; // orderIORatio
        orderStackAlice[1] = 1e18; // orderOutputMax

        uint256[] memory orderStackBob = new uint256[](2);
        orderStackBob[0] = bobIORatio; // orderIORatio
        orderStackBob[1] = 1e18; // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                1e18,
                1e18,
                expression,
                orderStackAlice,
                orderStackBob,
                0,
                0,
                0,
                0,
                stdError.arithmeticError
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioAliceOnly(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) external {
        uint256 aliceAmount = 2e18;
        uint256 bobAmount = 3e18;

        uint256[] memory orderStackAlice = new uint256[](2);
        orderStackAlice[0] = 0; // Zero orderIORatio
        orderStackAlice[1] = 0.5e18; // orderOutputMax

        uint256[] memory orderStackBob = new uint256[](2);
        orderStackBob[0] = 1e18; // Nonzero orderIORatio
        orderStackBob[1] = 0.5e18; // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                aliceAmount,
                bobAmount,
                expression,
                orderStackAlice,
                orderStackBob,
                0.5e18,
                0.5e18,
                0,
                0.5e18,
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioBobOnly(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) external {
        uint256 aliceAmount = 2e18;
        uint256 bobAmount = 3e18;

        uint256[] memory orderStackAlice = new uint256[](2);
        orderStackAlice[0] = 1e18; // Zero orderIORatio
        orderStackAlice[1] = 0.5e18; // orderOutputMax

        uint256[] memory orderStackBob = new uint256[](2);
        orderStackBob[0] = 0; // Nonzero orderIORatio
        orderStackBob[1] = 0.5e18; // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                aliceAmount,
                bobAmount,
                expression,
                orderStackAlice,
                orderStackBob,
                0.5e18,
                0.5e18,
                0.5e18,
                0,
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioAliceAndBob(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) external {
        uint256 aliceAmount = 2e18;
        uint256 bobAmount = 3e18;

        // Mock the interpreter.eval for Alice and Bob orders with zero ratio
        uint256[] memory orderStackAlice = new uint256[](2);
        orderStackAlice[0] = 0; // Zero orderIORatio
        orderStackAlice[1] = 5e17; // orderOutputMax

        uint256[] memory orderStackBob = new uint256[](2);

        orderStackBob[0] = 0; // Zero orderIORatio
        orderStackBob[1] = 5e17; // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                aliceAmount,
                bobAmount,
                expression,
                orderStackAlice,
                orderStackBob,
                0.5e18,
                0.5e18,
                0,
                0,
                ""
            )
        );
    }
}
