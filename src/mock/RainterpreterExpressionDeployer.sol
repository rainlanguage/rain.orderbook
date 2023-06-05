// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "rain.interface.interpreter/IExpressionDeployerV1.sol";

contract RainterpreterExpressionDeployer is IExpressionDeployerV1{  

    IInterpreterV1 public interpreter;
    IInterpreterStoreV1 public store;

    constructor(
        IInterpreterV1 interpreter_ ,
        IInterpreterStoreV1 store_
    ) {
        interpreter = interpreter_ ;
        store = store_ ;
    }


    /// @inheritdoc IExpressionDeployerV1
    function deployExpression(
        bytes[] memory sources_,
        uint256[] memory constants_,
        uint256[] memory minOutputs_
    ) external returns (IInterpreterV1, IInterpreterStoreV1, address) {
        return (interpreter, store, address(0));
    }

}