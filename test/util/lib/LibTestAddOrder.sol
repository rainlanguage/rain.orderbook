// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity ^0.8.19;

import {META_MAGIC_NUMBER_V1} from "rain.metadata/lib/LibMeta.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {OrderConfigV3, OrderV3, IO} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IInterpreterV3, SourceIndexV2} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/deprecated/IExpressionDeployerV3.sol";
import {EvaluableV3} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";
import {HANDLE_IO_ENTRYPOINT} from "src/concrete/ob/OrderBook.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";

library LibTestAddOrder {
    /// A little boilerplate to make it easier to build the order that we expect
    /// for a given order config.
    function expectedOrder(address owner, OrderConfigV3 memory config)
        internal
        pure
        returns (OrderV3 memory, bytes32)
    {
        OrderV3 memory order = OrderV3(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        return (order, LibOrder.hash(order));
    }

    /// Valid config has a few requirements. Mutates the config in place.
    /// Anything that doesn't meet the requirements will just be set to 0 values
    /// as this is faster than forcing the fuzzer to rebuild with assume.
    function conformConfig(OrderConfigV3 memory config, IInterpreterV3 interpreter, IInterpreterStoreV2 store)
        internal
        pure
    {
        if (config.meta.length > 0) {
            // This is a bit of a hack, but it's the easiest way to get a valid
            // meta document.
            config.meta = abi.encodePacked(META_MAGIC_NUMBER_V1, config.meta);
        }
        config.evaluable.interpreter = interpreter;
        config.evaluable.store = store;
        if (config.validInputs.length == 0) {
            config.validInputs = new IO[](1);
            config.validInputs[0] = IO(address(0), 0, 0);
        }
        if (config.validOutputs.length == 0) {
            config.validOutputs = new IO[](1);
            config.validOutputs[0] = IO(address(1), 0, 0);
        }
        if (config.validInputs[0].token == config.validOutputs[0].token) {
            config.validInputs[0].token = address(0);
            config.validOutputs[0].token = address(1);
        }
        // Taken from parser for "_ _:1e18 1e18;:;".
        config.evaluable.bytecode = hex"020000000c02020002010000000100000000000000";
    }
}
