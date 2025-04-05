// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, stdError} from "forge-std/Test.sol";

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    IOV2,
    ClearConfigV2,
    EvaluableV4,
    SignedContextV1,
    IInterpreterV4,
    TaskV2,
    Float
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotOrderOwner, StackItem} from "src/concrete/ob/OrderBook.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {StateNamespace} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {LibFixedPointDecimalArithmeticOpenZeppelin} from
    "rain.math.fixedpoint/lib/LibFixedPointDecimalArithmeticOpenZeppelin.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {LibDecimalFloat, PackedFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
    using LibFixedPointDecimalArithmeticOpenZeppelin for uint256;
    using Math for uint256;
    using LibDecimalFloat for Float;

    /// Make a deposit to the OB mocking the internal transferFrom call.
    function _depositInternal(address depositor, address token, bytes32 vaultId, Float memory amount) internal {
        vm.prank(depositor);
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(true)
        );
        iOrderbook.deposit3(address(token), vaultId, amount, new TaskV2[](0));

        Float memory balance = iOrderbook.vaultBalance2(depositor, token, vaultId);

        assertTrue(balance.eq(amount));
    }

    function conformBasicConfig(OrderConfigV4 memory aliceConfig, OrderConfigV4 memory bobConfig) internal view {
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

        aliceConfig.meta = "";
        bobConfig.meta = "";
    }

    struct CheckClear {
        address alice;
        OrderConfigV4 aliceConfig;
        address bob;
        OrderConfigV4 bobConfig;
        address bountyBot;
        bytes32 aliceBountyVaultId;
        bytes32 bobBountyVaultId;
        Float aliceAmount;
        Float bobAmount;
        bytes expression;
        StackItem[] orderStackAlice;
        StackItem[] orderStackBob;
        Float expectedAliceOutput;
        Float expectedBobOutput;
        Float expectedAliceInput;
        Float expectedBobInput;
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

        {
            Float memory aliceInputBalance = iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validInputs[0].token, clear.aliceConfig.validInputs[0].vaultId
            );
            assertTrue(LibDecimalFloat.eq(aliceInputBalance.signedCoefficient, aliceInputBalance.exponent, 0, 0));
        }
        {
            Float memory aliceOutputBalance = iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
            );
            assertTrue(aliceOutputBalance.eq(clear.aliceAmount));
        }
        {
            Float memory bobInputBalance = iOrderbook.vaultBalance2(
                clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId
            );
            assertTrue(LibDecimalFloat.eq(bobInputBalance.signedCoefficient, bobInputBalance.exponent, 0, 0));
        }
        {
            Float memory bobOutputBalance = iOrderbook.vaultBalance2(
                clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
            );
            assertTrue(bobOutputBalance.eq(clear.bobAmount), "Bob output balance should be equal to bob amount");
        }
        {
            Float memory aliceBountyBalance = iOrderbook.vaultBalance2(
                clear.bountyBot, clear.aliceConfig.validOutputs[0].token, clear.aliceBountyVaultId
            );
            assertTrue(LibDecimalFloat.eq(aliceBountyBalance.signedCoefficient, aliceBountyBalance.exponent, 0, 0));
        }
        {
            Float memory bobBountyBalance =
                iOrderbook.vaultBalance2(clear.bountyBot, clear.bobConfig.validOutputs[0].token, clear.bobBountyVaultId);
            assertTrue(LibDecimalFloat.eq(bobBountyBalance.signedCoefficient, bobBountyBalance.exponent, 0, 0));
        }

        {
            {
                bytes memory call = abi.encodeWithSelector(
                    IInterpreterV4.eval4.selector,
                    clear.aliceConfig.evaluable.store,
                    LibNamespace.qualifyNamespace(
                        StateNamespace.wrap(uint256(uint160(clear.alice))), address(iOrderbook)
                    )
                );

                vm.mockCall(address(iInterpreter), call, abi.encode(clear.orderStackAlice, new uint256[](0)));

                call = abi.encodeWithSelector(
                    IInterpreterV4.eval4.selector,
                    clear.bobConfig.evaluable.store,
                    LibNamespace.qualifyNamespace(StateNamespace.wrap(uint256(uint160(clear.bob))), address(iOrderbook))
                );

                vm.mockCall(address(iInterpreter), call, abi.encode(clear.orderStackBob, new uint256[](0)));
            }

            OrderV4 memory aliceOrder;
            OrderV4 memory bobOrder;
            {
                (aliceOrder,) = addOrderWithChecks(clear.alice, clear.aliceConfig, clear.expression);
                (bobOrder,) = addOrderWithChecks(clear.bob, clear.bobConfig, clear.expression);
            }

            ClearConfigV2 memory configClear = ClearConfigV2({
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
            iOrderbook.clear3(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
        }

        {
            Float memory aliceOutputBalance = iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
            );
            // Float memory expectedAliceOutput = Float({
            //     signedCoefficient: clear.expectedAliceOutput,
            //     exponent: 0
            // });
        }

        assertTrue(
            iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
            ).eq(clear.aliceAmount.sub(clear.expectedAliceOutput)),
            "Alice output vault"
        );
        assertTrue(
            iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validInputs[0].token, clear.aliceConfig.validInputs[0].vaultId
            ).eq(clear.expectedAliceInput),
            "Alice input vault"
        );

        assertTrue(
            iOrderbook.vaultBalance2(
                clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
            ).eq(clear.bobAmount.sub(clear.expectedBobOutput)),
            "Bob output vault"
        );
        assertTrue(
            iOrderbook.vaultBalance2(
                clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId
            ).eq(clear.expectedBobInput),
            "Bob input vault"
        );

        assertTrue(
            iOrderbook.vaultBalance2(clear.bountyBot, clear.aliceConfig.validOutputs[0].token, clear.aliceBountyVaultId)
                .eq(clear.expectedAliceOutput.sub(clear.expectedBobInput)),
            "Alice bounty"
        );
        assertTrue(
            iOrderbook.vaultBalance2(clear.bountyBot, clear.aliceConfig.validInputs[0].token, clear.aliceBountyVaultId)
                .eq(Float(0, 0)),
            "Alice bounty input"
        );
        assertTrue(
            iOrderbook.vaultBalance2(clear.bountyBot, clear.bobConfig.validOutputs[0].token, clear.bobBountyVaultId).eq(
                clear.expectedBobOutput.sub(clear.expectedAliceInput)
            ),
            "Bob bounty"
        );
        assertTrue(
            iOrderbook.vaultBalance2(clear.bountyBot, clear.bobConfig.validInputs[0].token, clear.bobBountyVaultId).eq(
                Float(0, 0)
            ),
            "Bob bounty input"
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearSimple(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float memory aliceAmount = Float(2, 0);
        Float memory bobAmount = Float(2, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(PackedFloat.unwrap(Float(0.99e18, -18).pack())); // orderIORatio
        orderStackAlice[1] = StackItem.wrap(PackedFloat.unwrap(Float(0.5e18, -18).pack())); // orderOutputMax

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
                Float(0.5e18, -18),
                Float(0.5e18, -18),
                Float(0.495e18, -18),
                Float(0.495e18, -18),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatio(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId,
        uint256 aliceIORatio18,
        uint256 bobIORatio18
    ) external {
        // 0 tested separately.
        aliceIORatio18 = bound(aliceIORatio18, 1, 1e18);
        bobIORatio18 = bound(bobIORatio18, 1e18, uint256(1e18).fixedPointDiv(aliceIORatio18, Math.Rounding.Down));

        Float memory aliceIORatio = Float(int256(aliceIORatio18), -18);
        Float memory bobIORatio = Float(int256(bobIORatio18), -18);

        Float memory aliceOutput = Float(1, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(PackedFloat.unwrap(aliceIORatio.pack())); // orderIORatio
        orderStackAlice[1] = StackItem.wrap(PackedFloat.unwrap(aliceOutput.pack())); // orderOutputMax

        StackItem[] memory orderStackBob = new StackItem[](2);
        orderStackBob[0] = StackItem.wrap(PackedFloat.unwrap(bobIORatio.pack())); // orderIORatio
        orderStackBob[1] = StackItem.wrap(PackedFloat.unwrap(Float(1e18, -18).pack())); // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                Float(1, 0),
                Float(1, 0),
                expression,
                orderStackAlice,
                orderStackBob,
                Float(1, 0),
                // Alice is outputting 1 so bob will output enough to match this
                // according to his own IO ratio.
                bobIORatio.inv().min(Float(1, 0)),
                // Expected input for Alice is aliceOutput * aliceIORatio
                aliceIORatio.multiply(aliceOutput),
                // Expected input for Bob is Alice's output in entirety, because
                // alice IO * bob IO <= 1 and Bob is the larger ratio.
                // As Bob's ratio is >= 1 he will have his input shrunk to match
                // Alice's output. This means in this case Bob's input will be
                // 1 always, as it either = 1 anyway or matches Alice's 1.
                Float(1, 0),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatioError(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId,
        uint256 aliceIORatio18,
        uint256 bobIORatio18
    ) external {
        aliceIORatio18 = bound(aliceIORatio18, 1e18 + 1, 1.1e18);
        bobIORatio18 = bound(bobIORatio18, 1e18 + 1, 1.1e18);

        Float memory aliceIORatio = Float(int256(aliceIORatio18), -18);
        Float memory bobIORatio = Float(int256(bobIORatio18), -18);

        Float memory aliceOutput = Float(1, 0);
        Float memory bobOutput = Float(1, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(PackedFloat.unwrap(aliceIORatio.pack())); // orderIORatio
        orderStackAlice[1] = StackItem.wrap(PackedFloat.unwrap(aliceOutput.pack())); // orderOutputMax

        StackItem[] memory orderStackBob = new StackItem[](2);
        orderStackBob[0] = StackItem.wrap(PackedFloat.unwrap(bobIORatio.pack())); // orderIORatio
        orderStackBob[1] = StackItem.wrap(PackedFloat.unwrap(bobOutput.pack())); // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                Float(1, 0),
                Float(1, 0),
                expression,
                orderStackAlice,
                orderStackBob,
                Float(0, 0),
                Float(0, 0),
                Float(0, 0),
                Float(0, 0),
                stdError.arithmeticError
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioAliceOnly(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float memory aliceAmount = Float(2e18, -18);
        Float memory bobAmount = Float(3e18, -18);

        Float memory aliceIORatio = Float(0, 0);
        Float memory bobIORatio = Float(1, 0);

        Float memory aliceOutputMax = Float(0.5e18, -18);
        Float memory bobOutputMax = Float(0.5e18, -18);

        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(PackedFloat.unwrap(aliceIORatio.pack()));
        orderStackAlice[1] = StackItem.wrap(PackedFloat.unwrap(aliceOutputMax.pack()));

        StackItem[] memory orderStackBob = new StackItem[](2);
        orderStackBob[0] = StackItem.wrap(PackedFloat.unwrap(bobIORatio.pack()));
        orderStackBob[1] = StackItem.wrap(PackedFloat.unwrap(bobOutputMax.pack()));

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
                Float(0.5e18, -18),
                Float(0.5e18, -18),
                Float(0, 0),
                Float(0.5e18, -18),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioBobOnly(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float memory aliceAmount = Float(2, 0);
        Float memory bobAmount = Float(3, 0);

        Float memory aliceIORatio = Float(1, 0);
        Float memory bobIORatio = Float(0, 0);

        Float memory aliceOutputMax = Float(0.5e18, -18);
        Float memory bobOutputMax = Float(0.5e18, -18);

        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(PackedFloat.unwrap(aliceIORatio.pack()));
        orderStackAlice[1] = StackItem.wrap(PackedFloat.unwrap(aliceOutputMax.pack()));

        StackItem[] memory orderStackBob = new StackItem[](2);
        orderStackBob[0] = StackItem.wrap(PackedFloat.unwrap(bobIORatio.pack()));
        orderStackBob[1] = StackItem.wrap(PackedFloat.unwrap(bobOutputMax.pack()));

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
                Float(0.5e18, -18),
                Float(0.5e18, -18),
                Float(0.5e18, -18),
                Float(0, 0),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioAliceAndBob(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float memory aliceAmount = Float(2, 0);
        Float memory bobAmount = Float(3, 0);

        Float memory aliceIORatio = Float(0, 0);
        Float memory bobIORatio = Float(0, 0);

        Float memory aliceOutputMax = Float(0.5e18, -18);
        Float memory bobOutputMax = Float(0.5e18, -18);

        // Mock the interpreter.eval for Alice and Bob orders with zero ratio
        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(PackedFloat.unwrap(aliceIORatio.pack()));
        orderStackAlice[1] = StackItem.wrap(PackedFloat.unwrap(aliceOutputMax.pack()));

        StackItem[] memory orderStackBob = new StackItem[](2);

        orderStackBob[0] = StackItem.wrap(PackedFloat.unwrap(bobIORatio.pack()));
        orderStackBob[1] = StackItem.wrap(PackedFloat.unwrap(bobOutputMax.pack()));

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
                Float(0.5e18, -18),
                Float(0.5e18, -18),
                Float(0, 0),
                Float(0, 0),
                ""
            )
        );
    }
}
