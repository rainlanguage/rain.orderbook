// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "rain.interface.interpreter/IInterpreterV1.sol";


contract Rainterpreter is IInterpreterV1 {
  
    function eval(
        IInterpreterStoreV1 store_,
        StateNamespace namespace_,
        EncodedDispatch dispatch_,
        uint256[][] memory context_
    ) external view returns (uint256[] memory, uint256[] memory) {
       
    } 

    function functionPointers() external view virtual returns (bytes memory) {
       
    }

}
