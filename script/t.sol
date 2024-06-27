
// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Script} from "forge-std/Script.sol";

contract MyTest is Script {
    function run() external {
        // command to run:
        // forge script ./script/t.sol:MyTest -vvvvv --fork-url <url> --fork-block-number <bn> --sender <addres> --offline
        address to = <address>;
        bytes memory _calldata = hex"";
        (bool success, bytes memory result) = to.call(_calldata);
        (success);
        (result);
    }
}