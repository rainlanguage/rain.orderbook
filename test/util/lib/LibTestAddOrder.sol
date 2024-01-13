// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import {META_MAGIC_NUMBER_V1} from "rain.metadata/lib/LibMeta.sol";
import {LibOrder} from "src/lib/LibOrder.sol";
import {OrderConfigV2, OrderV2, IO} from "src/interface/unstable/IOrderBookV3.sol";
import {IInterpreterV2, SourceIndexV2} from "rain.interpreter/interface/unstable/IInterpreterV2.sol";
import {IInterpreterStoreV1} from "rain.interpreter/interface/IInterpreterStoreV1.sol";
import {IExpressionDeployerV3} from "rain.interpreter/interface/unstable/IExpressionDeployerV3.sol";
import {EvaluableV2} from "rain.interpreter/interface/IInterpreterCallerV2.sol";
import {HANDLE_IO_ENTRYPOINT} from "src/concrete/OrderBook.sol";
import {LibBytecode} from "rain.interpreter/lib/bytecode/LibBytecode.sol";

library LibTestAddOrder {
    /// A little boilerplate to make it easier to build the order that we expect
    /// for a given order config.
    function expectedOrder(
        address owner,
        OrderConfigV2 memory config,
        IInterpreterV2 interpreter,
        IInterpreterStoreV1 store,
        address expression
    ) internal pure returns (OrderV2 memory, bytes32) {
        EvaluableV2 memory expectedEvaluable = EvaluableV2(interpreter, store, expression);
        OrderV2 memory order = OrderV2(
            owner,
            LibBytecode.sourceCount(config.evaluableConfig.bytecode) > 1
                && LibBytecode.sourceOpsCount(config.evaluableConfig.bytecode, SourceIndexV2.unwrap(HANDLE_IO_ENTRYPOINT))
                    > 0,
            expectedEvaluable,
            config.validInputs,
            config.validOutputs
        );
        return (order, LibOrder.hash(order));
    }

    /// Valid config has a few requirements. Mutates the config in place.
    /// Anything that doesn't meet the requirements will just be set to 0 values
    /// as this is faster than forcing the fuzzer to rebuild with assume.
    function conformConfig(OrderConfigV2 memory config, IExpressionDeployerV3 deployer) internal pure {
        if (config.meta.length > 0) {
            // This is a bit of a hack, but it's the easiest way to get a valid
            // meta document.
            config.meta = abi.encodePacked(META_MAGIC_NUMBER_V1, config.meta);
        }
        config.evaluableConfig.deployer = deployer;
        if (config.validInputs.length == 0) {
            config.validInputs = new IO[](1);
            config.validInputs[0] = IO(address(0), 0, 0);
        }
        if (config.validOutputs.length == 0) {
            config.validOutputs = new IO[](1);
            config.validOutputs[0] = IO(address(0), 0, 0);
        }
        // Taken from parser for "_ _:1e18 1e18;:;".
        config.evaluableConfig.bytecode = hex"020000000c02020002010000000100000000000000";
    }
}
