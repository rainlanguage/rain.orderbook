// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test, console2} from "lib/forge-std/src/Test.sol";
import {IMetaV1} from "lib/rain.metadata/src/IMetaV1.sol";
import {IExpressionDeployerV2} from "lib/rain.interpreter/src/interface/unstable/IExpressionDeployerV2.sol";
import {IOrderBookV3Stub} from "test/util/abstract/IOrderBookV3Stub.sol";
import {IInterpreterV1} from "rain.interpreter/src/interface/IInterpreterV1.sol";
import {IInterpreterStoreV1} from "rain.interpreter/src/interface/IInterpreterStoreV1.sol";
import {IOrderBookV3, OrderConfigV2, Order} from "src/interface/unstable/IOrderBookV3.sol";
import {IERC20} from "openzeppelin-contracts/contracts/interfaces/IERC20.sol";
import {OrderBook} from "src/concrete/OrderBook.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {DeployerDiscoverableMetaV2ConstructionConfig} from "rain.interpreter/src/abstract/DeployerDiscoverableMetaV2.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";

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
/// Implements IOrderBookV3 so that it has access to all the relevant events.
abstract contract OrderBookExternalMockTest is Test, IMetaV1, IOrderBookV3Stub {
    IInterpreterV1 immutable iInterpreter;
    IInterpreterStoreV1 immutable iStore;
    IExpressionDeployerV2 immutable iDeployer;
    IOrderBookV3 immutable iOrderbook;
    IERC20 immutable iToken0;
    IERC20 immutable iToken1;

    constructor() {
        vm.pauseGasMetering();
        iInterpreter = IInterpreterV1(address(uint160(uint256(keccak256("interpreter.rain.test")))));
        vm.etch(address(iInterpreter), REVERTING_MOCK_BYTECODE);
        iStore = IInterpreterStoreV1(address(uint160(uint256(keccak256("store.rain.test")))));
        vm.etch(address(iStore), REVERTING_MOCK_BYTECODE);
        iDeployer = IExpressionDeployerV2(address(uint160(uint256(keccak256("deployer.rain.test")))));
        // All non-mocked calls will revert.
        vm.etch(address(iDeployer), REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV2.deployExpression.selector),
            abi.encode(iInterpreter, iStore, address(0))
        );
        bytes memory meta = vm.readFileBinary(ORDER_BOOK_META_PATH);
        console2.log("OrderBookExternalMockTest meta hash:");
        console2.logBytes(abi.encodePacked(keccak256(meta)));
        iOrderbook =
            IOrderBookV3(address(new OrderBook(DeployerDiscoverableMetaV2ConstructionConfig(address(iDeployer), meta))));

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.resumeGasMetering();
    }

    /// Boilerplate to add an order with a mocked deployer and checks events and
    /// storage accesses.
    function addOrderWithChecks(address owner, OrderConfigV2 memory config, address expression)
        internal
        returns (Order memory, bytes32)
    {
        config.evaluableConfig.deployer = iDeployer;
        (Order memory order, bytes32 orderHash) =
            LibTestAddOrder.expectedOrder(owner, config, iInterpreter, iStore, expression);
        assertTrue(!iOrderbook.orderExists(orderHash));
        vm.mockCall(
            address(iDeployer),
            abi.encodeWithSelector(IExpressionDeployerV2.deployExpression.selector),
            abi.encode(iInterpreter, iStore, expression)
        );
        vm.expectEmit(false, false, false, true);
        emit AddOrder(owner, iDeployer, order, orderHash);
        if (config.meta.length > 0) {
            vm.expectEmit(false, false, true, false);
            // The subject of the meta is the order hash.
            emit MetaV1(owner, uint256(orderHash), config.meta);
        }
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        assertTrue(iOrderbook.addOrder(config));
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
            abi.encodeWithSelector(IExpressionDeployerV2.deployExpression.selector),
            abi.encode(iInterpreter, iStore, expression)
        );
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        assertFalse(iOrderbook.addOrder(config));
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
    function removeOrderWithChecks(address owner, Order memory order) internal {
        bytes32 orderHash = LibOrder.hash(order);
        // This check assumes the order exists before we try to remove it.
        assertTrue(iOrderbook.orderExists(orderHash));
        vm.expectEmit(false, false, false, true);
        emit RemoveOrder(owner, order, orderHash);
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        // An order was removed so this is true as there is a state change.
        assertTrue(iOrderbook.removeOrder(order));
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
        assertFalse(iOrderbook.removeOrder(order));
        assertEq(vm.getRecordedLogs().length, 0);
        (reads, writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
        assertFalse(iOrderbook.orderExists(orderHash));
    }
}
