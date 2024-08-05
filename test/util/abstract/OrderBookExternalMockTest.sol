// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/deprecated/IExpressionDeployerV3.sol";
import {IMetaV1} from "rain.metadata/lib/LibMeta.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IOrderBookV4Stub} from "test/util/abstract/IOrderBookV4Stub.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {IOrderBookV4, OrderConfigV3, OrderV3, ActionV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {OrderBook} from "src/concrete/ob/OrderBook.sol";
import {EvaluableV3} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";

/// @title OrderBookExternalTest
/// Abstract contract that performs common setup needed for testing an orderbook
/// from its external interface.
///
/// Notably:
/// - Deploys a real orderbook contract with correct meta.
/// - Deploys several mockable token contracts.
/// - Deploys a mockable deployer contract for a DISpair.
///
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Implements IOrderBookV4 so that it has access to all the relevant events.
abstract contract OrderBookExternalMockTest is Test, IMetaV1, IOrderBookV4Stub {
    IInterpreterV3 immutable iInterpreter;
    IInterpreterStoreV2 immutable iStore;
    IExpressionDeployerV3 immutable iDeployer;
    IOrderBookV4 immutable iOrderbook;
    IERC20 immutable iToken0;
    IERC20 immutable iToken1;

    constructor() {
        vm.pauseGasMetering();
        iInterpreter = IInterpreterV3(address(uint160(uint256(keccak256("interpreter.rain.test")))));
        vm.etch(address(iInterpreter), REVERTING_MOCK_BYTECODE);
        iStore = IInterpreterStoreV2(address(uint160(uint256(keccak256("store.rain.test")))));
        vm.etch(address(iStore), REVERTING_MOCK_BYTECODE);
        iDeployer = IExpressionDeployerV3(address(uint160(uint256(keccak256("deployer.rain.test")))));
        // All non-mocked calls will revert.
        vm.etch(address(iDeployer), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(iInterpreter, iStore, address(0), "00020000")
        );
        iOrderbook = IOrderBookV4(address(new OrderBook()));

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.resumeGasMetering();
    }

    /// Boilerplate to add an order with a mocked deployer and checks events and
    /// storage accesses.
    function addOrderWithChecks(address owner, OrderConfigV3 memory config, bytes memory expression)
        internal
        returns (OrderV3 memory, bytes32)
    {
        (OrderV3 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(owner, config);
        assertTrue(!iOrderbook.orderExists(orderHash));
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(iInterpreter, iStore, expression, hex"00020000")
        );
        vm.expectEmit(false, false, false, true);
        emit AddOrderV2(owner, orderHash, order);
        if (config.meta.length > 0) {
            vm.expectEmit(false, false, true, false);
            // The subject of the meta is the order hash.
            emit MetaV1(owner, uint256(orderHash), config.meta);
        }
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        assertTrue(iOrderbook.addOrder2(config, new ActionV1[](0)));
        // MetaV1 is NOT emitted if the meta is empty.
        assertEq(vm.getRecordedLogs().length, config.meta.length > 0 ? 2 : 1);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check, 1x for live write.
        assertEq(reads.length, 5);
        // 2x for reentrancy guard, 1x for live write.
        assertEq(writes.length, 3);
        assertTrue(iOrderbook.orderExists(orderHash));

        // Adding the same order again MUST NOT change state. This MAY be
        // impossible to encounter for a real expression deployer, as the
        // deployer MAY NOT return the same address twice, but it is possible to
        // mock.
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(iInterpreter, iStore, expression, hex"00020000")
        );
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        assertFalse(iOrderbook.addOrder2(config, new ActionV1[](0)));
        assertEq(vm.getRecordedLogs().length, 0);
        (reads, writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
        assertTrue(iOrderbook.orderExists(orderHash));

        return (order, orderHash);
    }

    /// Boilerplate to remove an order with a mocked deployer and checks events
    /// and storage accesses.
    function removeOrderWithChecks(address owner, OrderV3 memory order) internal {
        bytes32 orderHash = LibOrder.hash(order);
        // This check assumes the order exists before we try to remove it.
        assertTrue(iOrderbook.orderExists(orderHash));
        vm.expectEmit(false, false, false, true);
        emit RemoveOrderV2(owner, orderHash, order);
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        // An order was removed so this is true as there is a state change.
        assertTrue(iOrderbook.removeOrder2(order, new ActionV1[](0)));
        assertEq(vm.getRecordedLogs().length, 1);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check, 1x for dead write.
        assertEq(reads.length, 5);
        // 2x for reentrancy guard, 1x for dead write.
        assertEq(writes.length, 3);
        assertFalse(iOrderbook.orderExists(orderHash));

        // Removing the same order again MUST NOT change state.
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        // There is no state change so this is false.
        assertFalse(iOrderbook.removeOrder2(order, new ActionV1[](0)));
        assertEq(vm.getRecordedLogs().length, 0);
        (reads, writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
        assertFalse(iOrderbook.orderExists(orderHash));
    }
}
