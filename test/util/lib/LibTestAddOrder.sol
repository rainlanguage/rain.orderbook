// SPDX-License-Identifier: CAL
pragma solidity ^0.8.19;

import "rain.metadata/LibMeta.sol";
import "src/interface/unstable/IOrderBookV3.sol";
import "src/lib/LibOrder.sol";
import "src/concrete/OrderBook.sol";

library LibTestAddOrder {
    /// A little boilerplate to make it easier to build the order that we expect
    /// for a given order config.
    function expectedOrder(
        address owner,
        OrderConfig memory config,
        IInterpreterV1 interpreter,
        IInterpreterStoreV1 store,
        address expression
    ) internal pure returns (Order memory, bytes32) {
        Evaluable memory expectedEvaluable = Evaluable(interpreter, store, expression);
        Order memory order = Order(
            owner,
            LibBytecode.sourceCount(config.evaluableConfig.bytecode) > 1
                && LibBytecode.sourceOpsLength(config.evaluableConfig.bytecode, SourceIndex.unwrap(HANDLE_IO_ENTRYPOINT))
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
    function conformConfig(OrderConfig memory config, IExpressionDeployerV2 deployer) internal pure {
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
        config.evaluableConfig.bytecode = hex"02000000040000000000000000";
    }
}
