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
import {NotOrderOwner, StackItem, NegativeBounty} from "src/concrete/ob/OrderBook.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {StateNamespace, EvalV4, SourceIndexV2} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {LibFixedPointDecimalArithmeticOpenZeppelin} from
    "rain.math.fixedpoint/lib/LibFixedPointDecimalArithmeticOpenZeppelin.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract MockInterpreter {
    StackItem[] internal sStack;

    constructor(StackItem[] memory stack) {
        sStack = stack;
    }

    function eval4(EvalV4 memory) external view returns (StackItem[] memory, bytes32[] memory) {
        return (sStack, new bytes32[](0));
    }
}

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
    using LibFixedPointDecimalArithmeticOpenZeppelin for uint256;
    using Math for uint256;
    using LibDecimalFloat for Float;

    /// Make a deposit to the OB mocking the internal transferFrom call.
    function _depositInternal(address depositor, address token, bytes32 vaultId, Float amount) internal {
        uint256 amount18 = LibDecimalFloat.toFixedDecimalLossless(amount, 18);
        vm.prank(depositor);
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount18),
            abi.encode(true)
        );
        iOrderbook.deposit3(address(token), vaultId, amount, new TaskV2[](0));

        Float balance = iOrderbook.vaultBalance2(depositor, token, vaultId);

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
            Float aliceInputBalance = iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validInputs[0].token, clear.aliceConfig.validInputs[0].vaultId
            );
            assertTrue(LibDecimalFloat.eq(aliceInputBalance, Float.wrap(0)));
        }
        {
            Float aliceOutputBalance = iOrderbook.vaultBalance2(
                clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
            );
            assertTrue(aliceOutputBalance.eq(clear.aliceAmount));
        }
        {
            Float bobInputBalance = iOrderbook.vaultBalance2(
                clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId
            );
            assertTrue(LibDecimalFloat.eq(bobInputBalance, Float.wrap(0)));
        }
        {
            Float bobOutputBalance = iOrderbook.vaultBalance2(
                clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
            );
            assertTrue(bobOutputBalance.eq(clear.bobAmount), "Bob output balance should be equal to bob amount");
        }
        {
            Float aliceBountyBalance = iOrderbook.vaultBalance2(
                clear.bountyBot, clear.aliceConfig.validOutputs[0].token, clear.aliceBountyVaultId
            );
            assertTrue(LibDecimalFloat.eq(aliceBountyBalance, Float.wrap(0)));
        }
        {
            Float bobBountyBalance =
                iOrderbook.vaultBalance2(clear.bountyBot, clear.bobConfig.validOutputs[0].token, clear.bobBountyVaultId);
            assertTrue(LibDecimalFloat.eq(bobBountyBalance, Float.wrap(0)));
        }

        {
            {
                MockInterpreter aliceInterpreter = new MockInterpreter(clear.orderStackAlice);
                MockInterpreter bobInterpreter = new MockInterpreter(clear.orderStackBob);
                clear.aliceConfig.evaluable.interpreter = IInterpreterV4(address(aliceInterpreter));
                clear.bobConfig.evaluable.interpreter = IInterpreterV4(address(bobInterpreter));
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
                .isZero(),
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
                LibDecimalFloat.packLossless(0, 0)
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
        Float aliceAmount = LibDecimalFloat.packLossless(2, 0);
        Float bobAmount = LibDecimalFloat.packLossless(2, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(Float.unwrap(LibDecimalFloat.packLossless(0.99e18, -18))); // orderIORatio
        orderStackAlice[1] = StackItem.wrap(Float.unwrap(LibDecimalFloat.packLossless(0.5e18, -18))); // orderOutputMax

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
                LibDecimalFloat.packLossless(0.5e18, -18),
                LibDecimalFloat.packLossless(0.5e18, -18),
                LibDecimalFloat.packLossless(0.495e18, -18),
                LibDecimalFloat.packLossless(0.495e18, -18),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatioHappy(
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

        Float aliceIORatio = LibDecimalFloat.fromFixedDecimalLosslessPacked(aliceIORatio18, 18);
        Float bobIORatio = LibDecimalFloat.fromFixedDecimalLosslessPacked(bobIORatio18, 18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.expression = expression;
        checkClearStruct.aliceAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.bobAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(1, 0);
        // Alice is outputting 1 so bob will output enough to match this
        // according to his own IO ratio.
        checkClearStruct.expectedBobOutput = bobIORatio.inv().min(LibDecimalFloat.packLossless(1, 0));
        // Expected input for Alice is aliceOutput * aliceIORatio

        Float aliceOutput = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expectedAliceInput = aliceIORatio.multiply(aliceOutput);
        // Expected input for Bob is Alice's output in entirety, because
        // alice IO * bob IO <= 1 and Bob is the larger ratio.
        // As Bob's ratio is >= 1 he will have his input shrunk to match
        // Alice's output. This means in this case Bob's input will be
        // 1 always, as it either = 1 anyway or matches Alice's 1.
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expectedError = "";

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio)); // orderIORatio
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutput)); // orderOutputMax

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio)); // orderIORatio
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(LibDecimalFloat.packLossless(1e18, -18))); // orderOutputMax

        checkClear(checkClearStruct);
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

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.bobAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expression = expression;

        Float aliceIORatio = LibDecimalFloat.packLossless(int256(aliceIORatio18), -18);
        Float bobIORatio = LibDecimalFloat.packLossless(int256(bobIORatio18), -18);

        Float aliceOutput = LibDecimalFloat.packLossless(1, 0);
        Float bobOutput = LibDecimalFloat.packLossless(1, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio)); // orderIORatio
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutput)); // orderOutputMax

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio)); // orderIORatio
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutput)); // orderOutputMax

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedError = abi.encodeWithSelector(NegativeBounty.selector);

        checkClear(checkClearStruct);
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
        Float aliceAmount = LibDecimalFloat.packLossless(2e18, -18);
        Float bobAmount = LibDecimalFloat.packLossless(3e18, -18);

        Float aliceIORatio = LibDecimalFloat.packLossless(0, 0);
        Float bobIORatio = LibDecimalFloat.packLossless(1, 0);

        Float aliceOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);
        Float bobOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = aliceAmount;
        checkClearStruct.bobAmount = bobAmount;
        checkClearStruct.expression = expression;

        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio));
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutputMax));

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio));
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutputMax));

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0.5e18, -18);

        checkClearStruct.expectedError = "";

        checkClear(checkClearStruct);
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
        Float aliceAmount = Float.wrap(bytes32(uint256(2)));
        Float bobAmount = Float.wrap(bytes32(uint256(3)));

        Float aliceIORatio = LibDecimalFloat.packLossless(1, 0);
        Float bobIORatio = LibDecimalFloat.packLossless(0, 0);

        Float aliceOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);
        Float bobOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = aliceAmount;
        checkClearStruct.bobAmount = bobAmount;
        checkClearStruct.expression = expression;

        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio));
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutputMax));

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio));
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutputMax));

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedError = "";

        checkClear(checkClearStruct);
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
        Float aliceAmount = Float.wrap(bytes32(uint256(2)));
        Float bobAmount = Float.wrap(bytes32(uint256(3)));

        Float aliceIORatio = Float.wrap(bytes32(uint256(0)));
        Float bobIORatio = Float.wrap(bytes32(uint256(0)));

        Float aliceOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);
        Float bobOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = aliceAmount;
        checkClearStruct.bobAmount = bobAmount;
        checkClearStruct.expression = expression;

        // Mock the interpreter.eval for Alice and Bob orders with zero ratio
        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio));
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutputMax));

        checkClearStruct.orderStackBob = new StackItem[](2);

        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio));
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutputMax));

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedError = "";

        checkClear(checkClearStruct);
    }
}
