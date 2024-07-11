// SPDX-License-Identifier: CAL
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
    ActionV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotOrderOwner} from "src/concrete/ob/OrderBook.sol";

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
    function testClearSimple(
        address alice,
        OrderConfigV3 memory aliceConfig,
        address bob,
        OrderConfigV3 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) public {
        // Different accounts
        vm.assume(alice != bob);
        vm.assume(alice != bountyBot);
        vm.assume(bob != bountyBot);
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

        uint256 amount = 2e18;

        _depositInternal(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId, amount);
        _depositInternal(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId, amount);

        {
            // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
            // Produce the stack output for OB
            uint256[] memory orderStack = new uint256[](2);
            orderStack[0] = 1e18; // orderOutputMax
            orderStack[1] = 1e18; // orderIORatio
            vm.mockCall(
                address(iInterpreter),
                abi.encodeWithSelector(IInterpreterV3.eval3.selector),
                abi.encode(orderStack, new uint256[](0))
            );
        }

        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validInputs[0].token, aliceConfig.validInputs[0].vaultId), 0
        );
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId),
            amount
        );

        assertEq(iOrderbook.vaultBalance(bob, bobConfig.validInputs[0].token, bobConfig.validInputs[0].vaultId), 0);
        assertEq(
            iOrderbook.vaultBalance(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId), amount
        );

        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken0), aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken1), bobBountyVaultId), 0);

        {
            (OrderV3 memory aliceOrder, bytes32 aliceOrderHash) = addOrderWithChecks(alice, aliceConfig, expression);
            assertTrue(iOrderbook.orderExists(aliceOrderHash));

            (OrderV3 memory bobOrder, bytes32 bobOrderHash) = addOrderWithChecks(bob, bobConfig, expression);
            assertTrue(iOrderbook.orderExists(bobOrderHash));

            ClearConfig memory configClear = ClearConfig({
                aliceInputIOIndex: 0,
                aliceOutputIOIndex: 0,
                bobInputIOIndex: 0,
                bobOutputIOIndex: 0,
                aliceBountyVaultId: aliceBountyVaultId,
                bobBountyVaultId: bobBountyVaultId
            });

            vm.prank(bountyBot);
            iOrderbook.clear2(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
        }

        // After the clear alice and bob have traded 1e18 tokens, so they each
        // have 1e18 of each token. The bounty is 0 in this case.
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validInputs[0].token, aliceConfig.validInputs[0].vaultId), 1e18
        );
        assertEq(
            iOrderbook.vaultBalance(alice, aliceConfig.validOutputs[0].token, aliceConfig.validOutputs[0].vaultId), 1e18
        );

        assertEq(iOrderbook.vaultBalance(bob, bobConfig.validInputs[0].token, bobConfig.validInputs[0].vaultId), 1e18);
        assertEq(iOrderbook.vaultBalance(bob, bobConfig.validOutputs[0].token, bobConfig.validOutputs[0].vaultId), 1e18);

        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken0), aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken1), bobBountyVaultId), 0);
    }

    /// Make a deposit to the OB mocking the internal transferFrom call.
    function _depositInternal(address depositor, address token, uint256 vaultId, uint256 amount) internal {
        vm.prank(depositor);
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(true)
        );
        iOrderbook.deposit2(token, vaultId, amount, new ActionV1[](0));

        assertEq(iOrderbook.vaultBalance(depositor, token, vaultId), amount);
    }
}
