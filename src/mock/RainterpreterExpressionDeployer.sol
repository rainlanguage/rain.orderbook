// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "rain.interface.interpreter/IExpressionDeployerV1.sol";

contract RainterpreterExpressionDeployer is IExpressionDeployerV1{  



    /// @inheritdoc IExpressionDeployerV1
    function deployExpression(
        bytes[] memory sources_,
        uint256[] memory constants_,
        uint256[] memory minOutputs_
    ) external returns (IInterpreterV1, IInterpreterStoreV1, address) {
        
    }

}