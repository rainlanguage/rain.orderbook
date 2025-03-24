// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {META_MAGIC_NUMBER_V1} from "rain.metadata/lib/LibMeta.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {OrderConfigV4, OrderV4, IOV2} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IInterpreterV4, SourceIndexV2} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/unstable/IInterpreterStoreV3.sol";
import {EvaluableV3} from "rain.interpreter.interface/interface/IInterpreterCallerV3.sol";
import {HANDLE_IO_ENTRYPOINT} from "src/concrete/ob/OrderBook.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";

library LibTestAddOrder {
    /// A little boilerplate to make it easier to build the order that we expect
    /// for a given order config.
    function expectedOrder(address owner, OrderConfigV4 memory config)
        internal
        pure
        returns (OrderV4 memory, bytes32)
    {
        OrderV4 memory order = OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        return (order, LibOrder.hash(order));
    }

    /// Valid config has a few requirements. Mutates the config in place.
    /// Anything that doesn't meet the requirements will just be set to 0 values
    /// as this is faster than forcing the fuzzer to rebuild with assume.
    function conformConfig(OrderConfigV4 memory config, IInterpreterV4 interpreter, IInterpreterStoreV3 store)
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
            config.validInputs = new IOV2[](1);
            config.validInputs[0] = IOV2(address(0), 0);
        }
        if (config.validOutputs.length == 0) {
            config.validOutputs = new IOV2[](1);
            config.validOutputs[0] = IOV2(address(1), 0);
        }
        if (config.validInputs[0].token == config.validOutputs[0].token) {
            config.validInputs[0].token = address(0);
            config.validOutputs[0].token = address(1);
        }
        // Taken from parser for "_ _:1e18 1e18;:;".
        config.evaluable.bytecode = hex"020000000c02020002010000000100000000000000";
    }
}
