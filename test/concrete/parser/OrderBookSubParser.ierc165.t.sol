// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {OrderBookSubParser} from "src/concrete/parser/OrderBookSubParser.sol";
import {ISubParserV2} from "rain.interpreter.interface/interface/deprecated/ISubParserV2.sol";
import {IDescribedByMetaV1} from "rain.metadata/interface/IDescribedByMetaV1.sol";

contract OrderBookSubParserIERC165Test is Test {
    function testOrderBookSubParserIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(ISubParserV2).interfaceId);
        vm.assume(badInterfaceId != type(IDescribedByMetaV1).interfaceId);

        OrderBookSubParser subParser = new OrderBookSubParser();
        assertTrue(subParser.supportsInterface(type(IERC165).interfaceId));
        assertTrue(subParser.supportsInterface(type(ISubParserV2).interfaceId));
        assertTrue(subParser.supportsInterface(type(IDescribedByMetaV1).interfaceId));
        assertFalse(subParser.supportsInterface(badInterfaceId));
    }
}
