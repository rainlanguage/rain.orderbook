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
            config.evaluableConfig.sources.length > 1
                && config.evaluableConfig.sources[SourceIndex.unwrap(HANDLE_IO_ENTRYPOINT)].length > 0,
            expectedEvaluable,
            config.validInputs,
            config.validOutputs
        );
        return (order, LibOrder.hash(order));
    }

    /// Valid config has a few requirements. Mutates the config in place and
    /// returns a bool that indicates whether the config is otherwise valid. The
    /// bool should be passed directly to `vm.assume`.
    function conformConfig(OrderConfig memory config, IExpressionDeployerV1 deployer) internal pure returns (bool) {
        if (config.meta.length > 0) {
            // This is a bit of a hack, but it's the easiest way to get a valid
            // meta document.
            config.meta = abi.encodePacked(META_MAGIC_NUMBER_V1, config.meta);
        }
        config.evaluableConfig.deployer = deployer;
        return config.evaluableConfig.sources.length >= 2 && config.validInputs.length > 0
            && config.validOutputs.length > 0;
    }
}
