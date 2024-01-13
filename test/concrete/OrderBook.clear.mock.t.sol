// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test} from "lib/forge-std/src/Test.sol";

import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {OrderConfigV2, OrderV2, IO, ClearConfig} from "src/interface/unstable/IOrderBookV3.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {SignedContextV1} from "rain.interpreter/interface/IInterpreterCallerV2.sol";
import {IInterpreterV2} from "rain.interpreter/interface/unstable/IInterpreterV2.sol";
import {NotOrderOwner} from "src/concrete/OrderBook.sol";

/// @title OrderBookClearTest
/// Tests clearing an order.
contract OrderBookClearTest is OrderBookExternalMockTest {
    function testClearSimple(
        address alice,
        OrderConfigV2 memory aliceConfig,
        uint256 aliceVaultId,
        address bob,
        OrderConfigV2 memory bobConfig,
        uint256 bobVaultId,
        address expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) public {
        // Different accounts
        vm.assume(alice != bob);
        vm.assume(alice != bountyBot);
        vm.assume(bob != bountyBot);

        // -- Add two orders with similar IO tokens (swapped)
        // Add alice order with a input token (iToken0) and output token (iToken1)
        (OrderV2 memory aliceOrder, bytes32 aliceOrderHash) =
            _addOrderMockInternal(alice, aliceConfig, expression, iToken0, iToken1);
        assertTrue(iOrderbook.orderExists(aliceOrderHash));

        // Add bob order with a input token (iToken1) and output token (iToken0)
        (OrderV2 memory bobOrder, bytes32 bobOrderHash) =
            _addOrderMockInternal(bob, bobConfig, expression, iToken1, iToken0);
        assertTrue(iOrderbook.orderExists(bobOrderHash));

        // 2e18 tokens will be deposit for both (alice and bob)
        uint256 amount = 2e18;

        // Alice deposit his output token
        _depositInternal(alice, iToken1, aliceVaultId, amount);

        // Bob deposit his output token
        _depositInternal(bob, iToken0, bobVaultId, amount);

        // Since all the IO are just 1 length, the IOIndex will be zero (0).
        // And vaultIds for the clearer
        ClearConfig memory configClear = ClearConfig(0, 0, 0, 0, aliceBountyVaultId, bobBountyVaultId);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        uint256[] memory orderStack = new uint256[](2);
        orderStack[0] = 1e18; // orderOutputMax
        orderStack[1] = 1e18; // orderIORatio
        vm.mockCall(
            address(iInterpreter),
            abi.encodeWithSelector(IInterpreterV2.eval2.selector),
            abi.encode(orderStack, new uint256[](0))
        );

        // Clear the order using `bountyBot` address as caller clearer.
        vm.prank(bountyBot);
        iOrderbook.clear(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
    }

    /// Add an order using an owner (the caller) and modify the valid IOs to have
    /// just one valid IO from an input and output tokens.
    function _addOrderMockInternal(
        address owner,
        OrderConfigV2 memory config,
        address expression,
        IERC20 inputToken,
        IERC20 outputToken
    ) internal returns (OrderV2 memory, bytes32) {
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
        config.meta = new bytes(0);

        config.validInputs = _helperBuildIO(config.validInputs, address(inputToken), 18);
        config.validOutputs = _helperBuildIO(config.validOutputs, address(outputToken), 18);

        return addOrderWithChecks(owner, config, expression);
    }

    /// Make a deposit to the OB mocking the internal transferFrom call.
    function _depositInternal(address depositor, IERC20 token, uint256 vaultId, uint256 amount) internal {
        vm.prank(depositor);
        vm.mockCall(
            address(token),
            abi.encodeWithSelector(IERC20.transferFrom.selector, depositor, address(iOrderbook), amount),
            abi.encode(true)
        );
        iOrderbook.deposit(address(token), vaultId, amount);

        // Check that the vaultBalance was updated
        assertEq(iOrderbook.vaultBalance(depositor, address(token), vaultId), amount);
    }

    /// Edit a given IO array to have only one index, with a given token and decimal.
    /// This is useful to make matched Orders to do clears.
    function _helperBuildIO(IO[] memory io, address newToken, uint8 newDecimals) internal pure returns (IO[] memory) {
        IO[] memory ioAux = new IO[](1);

        ioAux[0] = io[0];
        ioAux[0].token = newToken;
        ioAux[0].decimals = newDecimals;

        return ioAux;
    }
}
