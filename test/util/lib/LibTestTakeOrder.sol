// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {OrderV4, TakeOrderConfigV4, TakeOrdersConfigV5} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/deprecated/v1/IInterpreterCallerV2.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

library LibTestTakeOrder {
    /// Extract OrderV4 from the first log entry emitted by addOrder4.
    function extractOrderFromLogs(Vm.Log[] memory entries) internal pure returns (OrderV4 memory) {
        (,, OrderV4 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV4));
        return order;
    }

    /// Wrap a single order into a TakeOrderConfigV4 array with default IO
    /// indices (0, 0) and empty signed context.
    function wrapSingle(OrderV4 memory order) internal pure returns (TakeOrderConfigV4[] memory) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));
        return orders;
    }

    /// Build a TakeOrdersConfigV5 with standard defaults: minimumIO=0,
    /// maximumIO=max, maximumIORatio=max, IOIsInput=true, data="".
    function defaultTakeConfig(TakeOrderConfigV4[] memory orders) internal pure returns (TakeOrdersConfigV5 memory) {
        return TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: orders,
            data: ""
        });
    }
}
