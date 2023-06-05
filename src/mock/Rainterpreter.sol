// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "sol.lib.datacontract/LibDataContract.sol";
import "sol.lib.memory/LibStackPointer.sol";
import "rain.interface.interpreter/LibEncodedDispatch.sol";
import "rain.lib.memkv/LibMemoryKV.sol";
import "rain.interface.interpreter/IInterpreterStoreV1.sol";
import "rain.lib.interpreter/LibInterpreterStateDataContract.sol";
import "rain.lib.interpreter/LibNamespace.sol";
import "sol.lib.memory/LibUint256Array.sol";
import "rain.lib.interpreter/LibEval.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol"; 
import "rain.lib.typecast/LibCast.sol"; 



contract Rainterpreter is IInterpreterV1 {
    using LibStackPointer for Pointer;
    using LibStackPointer for uint256[];
    using LibUint256Array for uint256[];
    using LibEval for InterpreterState;
    using LibNamespace for StateNamespace;
    using LibInterpreterStateDataContract for bytes;
    using LibCast for function(InterpreterState memory, Operand, Pointer)
        view
        returns (Pointer)[];
    using Math for uint256;
    using LibMemoryKV for MemoryKV; 
   
    function eval(
        IInterpreterStoreV1 store_,
        StateNamespace namespace_,
        EncodedDispatch dispatch_,
        uint256[][] memory context_
    ) external view returns (uint256[] memory, uint256[] memory) {
        // Decode the dispatch.
        (
            address expression_,
            SourceIndex sourceIndex_,
            uint256 maxOutputs_
        ) = LibEncodedDispatch.decode(dispatch_);

        // Build the interpreter state from the onchain expression.
        InterpreterState memory state_ = LibDataContract
            .read(expression_)
            .unsafeDeserialize();
        state_.stateKV = MemoryKV.wrap(0);
        state_.namespace = namespace_.qualifyNamespace();
        state_.store = store_;
        state_.context = context_;

        // Eval the expression and return up to maxOutputs_ from the final stack.
        Pointer stackTop_ = state_.eval(sourceIndex_, state_.stackBottom);
        uint256 stackLength_ = state_.stackBottom.unsafeToIndex(stackTop_);
        (, uint256[] memory tail_) = stackTop_.unsafeList(
            stackLength_.min(maxOutputs_)
        );
        return (tail_, state_.stateKV.toUint256Array());
    } 

    function functionPointers() external view virtual returns (bytes memory) {
        return hex"00";
    }

}
