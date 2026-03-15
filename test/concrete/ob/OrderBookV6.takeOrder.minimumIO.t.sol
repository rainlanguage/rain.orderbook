// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    OrderV4,
    TakeOrdersConfigV5,
    IOV2,
    EvaluableV4,
    OrderConfigV4,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {MinimumIO} from "../../../src/concrete/ob/OrderBookV6.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";

/// When the total taker IO is less than the configured minimumIO,
/// takeOrders4 must revert with MinimumIO(minimumIO, actualIO).
contract OrderBookV6TakeOrderMinimumIOTest is OrderBookV6ExternalRealTest {
    function testTakeOrderMinimumIORevert() external {
        address alice = address(uint160(uint256(keccak256("alice.rain.test"))));
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));

        // Order outputs 1e-18 at ratio 1.
        bytes memory bytecode = iParserV2.parse2("_ _:1e-18 1;:;");
        IOV2[] memory inputs = new IOV2[](1);
        inputs[0] = IOV2(address(iToken0), bytes32(uint256(0x01)));
        IOV2[] memory outputs = new IOV2[](1);
        outputs[0] = IOV2(address(iToken1), bytes32(uint256(0x01)));

        EvaluableV4 memory evaluable = EvaluableV4(iInterpreter, iStore, bytecode);
        OrderConfigV4 memory orderConfig = OrderConfigV4(evaluable, inputs, outputs, bytes32(0), bytes32(0), "");

        // Deposit 1 token into alice's output vault so the order can fill.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook)),
            abi.encode(true)
        );
        vm.prank(alice);
        iOrderbook.deposit4(
            address(iToken1), bytes32(uint256(0x01)), LibDecimalFloat.packLossless(1, 0), new TaskV2[](0)
        );

        // Add the order.
        vm.prank(alice);
        vm.recordLogs();
        iOrderbook.addOrder4(orderConfig, new TaskV2[](0));
        OrderV4 memory order = LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());

        // Take with minimumIO = 1, but order only provides 1e-18.
        TakeOrdersConfigV5 memory takeConfig = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        takeConfig.minimumIO = LibDecimalFloat.packLossless(1, 0);

        vm.prank(bob);
        vm.expectRevert(
            abi.encodeWithSelector(
                MinimumIO.selector, LibDecimalFloat.packLossless(1, 0), LibDecimalFloat.packLossless(1, -18)
            )
        );
        iOrderbook.takeOrders4(takeConfig);
    }
}
