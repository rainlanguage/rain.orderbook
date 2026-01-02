// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {OrderBookV6SubParser} from "src/concrete/parser/OrderBookV6SubParser.sol";

contract OrderBookV6SubParserDescribedByMetaV1Test is Test {
    function testOrderBookV6SubParserDescribedByMetaV1Happy() external {
        bytes memory describedByMeta = vm.readFileBinary("meta/OrderBookV6SubParser.rain.meta");
        OrderBookV6SubParser subParser = new OrderBookV6SubParser();

        assertEq(keccak256(describedByMeta), subParser.describedByMetaV1());
    }
}
