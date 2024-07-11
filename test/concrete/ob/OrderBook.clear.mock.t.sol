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
        uint256 aliceVaultId,
        address bob,
        OrderConfigV3 memory bobConfig,
        uint256 bobVaultId,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) public {
        // Different accounts
        vm.assume(alice != bob);
        vm.assume(alice != bountyBot);
        vm.assume(bob != bountyBot);

        aliceConfig.evaluable.interpreter = iInterpreter;
        aliceConfig.evaluable.store = iStore;

        bobConfig.evaluable.interpreter = iInterpreter;
        bobConfig.evaluable.store = iStore;

        // -- Add two orders with similar IO tokens (swapped)
        // Add alice order with a input token (iToken0) and output token (iToken1)
        (OrderV3 memory aliceOrder, bytes32 aliceOrderHash) =
            _addOrderMockInternal(alice, aliceConfig, expression, iToken0, iToken1);
        assertTrue(iOrderbook.orderExists(aliceOrderHash));

        // Add bob order with a input token (iToken1) and output token (iToken0)
        (OrderV3 memory bobOrder, bytes32 bobOrderHash) =
            _addOrderMockInternal(bob, bobConfig, expression, iToken1, iToken0);
        assertTrue(iOrderbook.orderExists(bobOrderHash));

        // 2e18 tokens will be deposit for both (alice and bob)
        uint256 amount = 2e18;

        // Alice deposit his output token
        _depositInternal(alice, iToken1, aliceVaultId, amount);

        // Bob deposit his output token
        _depositInternal(bob, iToken0, bobVaultId, amount);

        // fails because deposit vault id differs from order io vault id
        checkVaultBalance(alice, aliceOrder, amount);
        checkVaultBalance(bob, bobOrder, amount);

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
            abi.encodeWithSelector(IInterpreterV3.eval3.selector),
            abi.encode(orderStack, new uint256[](0))
        );

        // Clear the order using `bountyBot` address as caller clearer.
        vm.prank(bountyBot);
        iOrderbook.clear2(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
    }

    /// Add an order using an owner (the caller) and modify the valid IOs to have
    /// just one valid IO from an input and output tokens.
    function _addOrderMockInternal(
        address owner,
        OrderConfigV3 memory config,
        bytes memory expression,
        IERC20 inputToken,
        IERC20 outputToken
    ) internal returns (OrderV3 memory, bytes32) {
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.evaluable.bytecode = hex"02000000040000000000000000";
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
        iOrderbook.deposit2(address(token), vaultId, amount, new ActionV1[](0));

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

    function checkVaultBalance(address owner, OrderV3 memory order, uint256 targetAmount) internal {
        assertEq(
            iOrderbook.vaultBalance(owner, order.validOutputs[0].token, order.validOutputs[0].vaultId), targetAmount
        );
    }
}

/// Same as above with fixed vault issue, but fails unexpectedly with over/underflow
contract OrderBookClearTestWithFixedVaultIssue is OrderBookExternalMockTest {
    function testClearSimpleWithFixedVaultIssue(
        address alice,
        OrderConfigV3 memory aliceConfig,
        uint256 aliceVaultId,
        address bob,
        OrderConfigV3 memory bobConfig,
        uint256 bobVaultId,
        bytes memory expression,
        address bountyBot,
        uint256 aliceBountyVaultId,
        uint256 bobBountyVaultId
    ) public {
        // Different accounts
        vm.assume(alice != bob);
        vm.assume(alice != bountyBot);
        vm.assume(bob != bountyBot);

        aliceConfig.evaluable.interpreter = iInterpreter;
        aliceConfig.evaluable.store = iStore;

        bobConfig.evaluable.interpreter = iInterpreter2;
        bobConfig.evaluable.store = iStore2;

        // -- Add two orders with similar IO tokens (swapped)
        // Add alice order with a input token (iToken0) and output token (iToken1)
        (OrderV3 memory aliceOrder, bytes32 aliceOrderHash) =
            _addOrderMockInternal(alice, aliceConfig, expression, iToken0, iToken1, aliceVaultId);
        assertTrue(iOrderbook.orderExists(aliceOrderHash));

        // Add bob order with a input token (iToken1) and output token (iToken0)
        (OrderV3 memory bobOrder, bytes32 bobOrderHash) =
            _addOrderMockInternal(bob, bobConfig, expression, iToken1, iToken0, bobVaultId);
        assertTrue(iOrderbook.orderExists(bobOrderHash));

        // 10e18 tokens will be deposit for both (alice and bob)
        uint256 amount = 20e18;

        // Alice deposit his output token
        _depositInternal(alice, iToken1, aliceVaultId, amount);

        // Bob deposit his output token
        _depositInternal(bob, iToken0, bobVaultId, amount);

        checkVaultBalance(alice, aliceOrder, amount);
        checkVaultBalance(bob, bobOrder, amount);

        // Since all the IO are just 1 length, the IOIndex will be zero (0).
        // And vaultIds for the clearer
        ClearConfig memory configClear = ClearConfig(0, 0, 0, 0, aliceBountyVaultId, bobBountyVaultId);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        // aliceOrder
        uint256[] memory orderStack = new uint256[](2);
        orderStack[0] = 10e18; // orderOutputMax
        orderStack[1] = 1e18; // orderIORatio
        vm.mockCall(
            address(iInterpreter),
            abi.encodeWithSelector(IInterpreterV3.eval3.selector),
            abi.encode(orderStack, new uint256[](0))
        );
        // bobOrder
        uint256[] memory orderStack2 = new uint256[](2);
        orderStack2[0] = 20e18; // orderOutputMax
        orderStack2[1] = 5e17; // orderIORatio
        vm.mockCall(
            address(iInterpreter2),
            abi.encodeWithSelector(IInterpreterV3.eval3.selector),
            abi.encode(orderStack2, new uint256[](0))
        );

        // all bot bounties should be zero at this point
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken0), aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken1), aliceBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken0), bobBountyVaultId), 0);
        assertEq(iOrderbook.vaultBalance(bountyBot, address(iToken1), bobBountyVaultId), 0);

        // Clear the order using `bountyBot` address as caller clearer.
        vm.prank(bountyBot);
        // this should clear correctly with 10 token0 bounty, however panics
        iOrderbook.clear2(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));

        // at least one the bot's bounty vaults should be greater than 0
        assertTrue(
            iOrderbook.vaultBalance(bountyBot, address(iToken0), aliceBountyVaultId) > 0
                || iOrderbook.vaultBalance(bountyBot, address(iToken1), aliceBountyVaultId) > 0
                || iOrderbook.vaultBalance(bountyBot, address(iToken0), bobBountyVaultId) > 0
                || iOrderbook.vaultBalance(bountyBot, address(iToken1), bobBountyVaultId) > 0
        );
    }

    /// Add an order using an owner (the caller) and modify the valid IOs to have
    /// just one valid IO from an input and output tokens.
    function _addOrderMockInternal(
        address owner,
        OrderConfigV3 memory config,
        bytes memory expression,
        IERC20 inputToken,
        IERC20 outputToken,
        uint256 outputVaultId
    ) internal returns (OrderV3 memory, bytes32) {
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.evaluable.bytecode = hex"02000000040000000000000000";
        config.meta = new bytes(0);

        config.validInputs = _helperBuildIO(config.validInputs, address(inputToken), 18, outputVaultId);
        config.validOutputs = _helperBuildIO(config.validOutputs, address(outputToken), 18, outputVaultId);

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
        iOrderbook.deposit2(address(token), vaultId, amount, new ActionV1[](0));

        // Check that the vaultBalance was updated
        assertEq(iOrderbook.vaultBalance(depositor, address(token), vaultId), amount);
    }

    /// Edit a given IO array to have only one index, with a given token and decimal.
    /// This is useful to make matched Orders to do clears.
    function _helperBuildIO(IO[] memory io, address newToken, uint8 newDecimals, uint256 newVaultId)
        internal
        pure
        returns (IO[] memory)
    {
        IO[] memory ioAux = new IO[](1);

        ioAux[0] = io[0];
        ioAux[0].token = newToken;
        ioAux[0].decimals = newDecimals;
        ioAux[0].vaultId = newVaultId;

        return ioAux;
    }

    function checkVaultBalance(address owner, OrderV3 memory order, uint256 targetAmount) internal {
        assertEq(
            iOrderbook.vaultBalance(owner, order.validOutputs[0].token, order.validOutputs[0].vaultId), targetAmount
        );
    }
}
