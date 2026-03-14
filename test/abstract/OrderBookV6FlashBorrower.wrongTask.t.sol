// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {
    GenericPoolOrderBookV6FlashBorrower,
    OrderBookV6ArbConfig
} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {WrongTask} from "../../src/abstract/OrderBookV6ArbCommon.sol";
import {
    OrderV4,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1,
    IOV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

contract OrderBookV6FlashBorrowerWrongTaskTest is ArbTest {
    function buildArb(OrderBookV6ArbConfig memory config) internal override returns (address payable) {
        return payable(address(new GenericPoolOrderBookV6FlashBorrower(config)));
    }

    function expression() internal pure override returns (bytes memory) {
        return hex"deadbeef";
    }

    constructor() ArbTest() {}

    /// arb4 MUST revert with WrongTask when the provided task does not match
    /// the task configured at construction.
    /// forge-config: default.fuzz.runs = 10
    function testArb4WrongTask(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        EvaluableV4 memory evaluable
    ) external {
        vm.assume(
            address(evaluable.interpreter) != address(iInterpreter) || evaluable.store != iInterpreterStore
                || keccak256(evaluable.bytecode) != keccak256(expression())
        );
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(WrongTask.selector));
        GenericPoolOrderBookV6FlashBorrower(iArb)
            .arb4(
                iOrderBook,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: ""
            }),
                "",
                TaskV2({evaluable: evaluable, signedContext: new SignedContextV1[](0)})
            );
    }
}
