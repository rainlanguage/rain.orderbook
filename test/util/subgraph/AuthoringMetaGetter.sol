// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "lib/openzeppelin-contracts/contracts/utils/Address.sol";
import "lib/rain.interpreter/src/lib/op/LibAllStandardOpsNP.sol";

/// @title AuthoringMetaGetter
/// A contract to obtain the current AuthoringMeta of the interpreter
contract AuthoringMetaGetter {
    function getAuthoringMeta() external pure returns (bytes memory) {
        return LibAllStandardOpsNP.authoringMeta();
    }
}