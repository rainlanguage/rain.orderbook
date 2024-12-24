// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

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

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
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

    struct DoClear {
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
    }

    function doClear(DoClear memory clear) internal {
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
            iOrderbook.clear2(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
        }
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

        doClear(
            DoClear(
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
                orderStackAlice
            )
        );

        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validInputs[0].token, aliceConfig.validInputs[0].vaultId),
            0.495e18
        );
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId),
            1.5e18
        );

        assertEq(
            iOrderbook.vaultBalance(bob, bobConfig.validInputs[0].token, bobConfig.validInputs[0].vaultId), 0.495e18
        );
        assertEq(
            iOrderbook.vaultBalance(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId), 1.5e18
        );

        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken0), aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken1), aliceBountyVaultId), 0.005e18);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken1), bobBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken0), bobBountyVaultId), 0.005e18);
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

        doClear(
            DoClear(
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
                orderStackBob
            )
        );

        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId),
            aliceAmount - 0.5e18
        );
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validInputs[0].token, aliceConfig.validInputs[0].vaultId), 0
        );
        assertEq(
            iOrderbook.vaultBalance(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId),
            bobAmount - 0.5e18
        );
        assertEq(iOrderbook.vaultBalance(bob, bobConfig.validInputs[0].token, bobConfig.validInputs[0].vaultId), 0.5e18);
        assertEq(iOrderbook.vaultBalance(bountyBot, aliceConfig.validOutputs[0].token, aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, bobConfig.validOutputs[0].token, bobBountyVaultId), 0.5e18);
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

        doClear(
            DoClear(
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
                orderStackBob
            )
        );

        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId),
            aliceAmount - 0.5e18
        );
        assertEq(
            iOrderbook.vaultBalance(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId),
            bobAmount - 0.5e18
        );
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validInputs[0].token, aliceConfig.validInputs[0].vaultId), 0.5e18
        );
        assertEq(iOrderbook.vaultBalance(bob, bobConfig.validInputs[0].token, bobConfig.validInputs[0].vaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, aliceConfig.validOutputs[0].token, aliceBountyVaultId), 0.5e18);
        assertEq(iOrderbook.vaultBalance(bountyBot, bobConfig.validOutputs[0].token, bobBountyVaultId), 0);
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

        doClear(
            DoClear(
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
                orderStackBob
            )
        );

        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId),
            aliceAmount - 0.5e18
        );
        assertEq(
            iOrderbook.vaultBalance(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId),
            bobAmount - 0.5e18
        );
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validInputs[0].token, aliceConfig.validInputs[0].vaultId), 0
        );
        assertEq(iOrderbook.vaultBalance(bob, bobConfig.validInputs[0].token, bobConfig.validInputs[0].vaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, aliceConfig.validOutputs[0].token, aliceBountyVaultId), 0.5e18);
        assertEq(iOrderbook.vaultBalance(bountyBot, bobConfig.validOutputs[0].token, bobBountyVaultId), 0.5e18);
    }
}
