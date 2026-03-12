// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {OrderBookV6} from "../../../src/concrete/ob/OrderBookV6.sol";
import {OrderBookV6SubParser} from "../../../src/concrete/parser/OrderBookV6SubParser.sol";
import {
    CREATION_CODE as ORDERBOOK_CREATION_CODE,
    RUNTIME_CODE as ORDERBOOK_RUNTIME_CODE,
    DEPLOYED_ADDRESS as ORDERBOOK_GENERATED_ADDRESS
} from "../../../src/generated/OrderBookV6.pointers.sol";
import {
    CREATION_CODE as SUB_PARSER_CREATION_CODE,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE,
    DEPLOYED_ADDRESS as SUB_PARSER_GENERATED_ADDRESS
} from "../../../src/generated/OrderBookV6SubParser.pointers.sol";

contract LibOrderBookDeployTest is Test {
    /// Deploying OrderBookV6 via Zoltu MUST produce the expected address and
    /// codehash.
    function testDeployAddressOrderBook() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(OrderBookV6).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);
    }

    /// Deploying OrderBookV6SubParser via Zoltu MUST produce the expected
    /// address and codehash.
    function testDeployAddressSubParser() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(OrderBookV6SubParser).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed OrderBookV6 MUST match the expected
    /// codehash constant.
    function testExpectedCodeHashOrderBook() external {
        OrderBookV6 ob = new OrderBookV6();
        assertEq(address(ob).codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed OrderBookV6SubParser MUST match the
    /// expected codehash constant.
    function testExpectedCodeHashSubParser() external {
        OrderBookV6SubParser subParser = new OrderBookV6SubParser();
        assertEq(address(subParser).codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
    }

    /// The precompiled creation code constant for OrderBookV6 MUST match the
    /// compiler's creation code.
    function testCreationCodeOrderBook() external pure {
        assertEq(keccak256(ORDERBOOK_CREATION_CODE), keccak256(type(OrderBookV6).creationCode));
    }

    /// The precompiled creation code constant for OrderBookV6SubParser MUST
    /// match the compiler's creation code.
    function testCreationCodeSubParser() external pure {
        assertEq(keccak256(SUB_PARSER_CREATION_CODE), keccak256(type(OrderBookV6SubParser).creationCode));
    }

    /// The precompiled runtime code constant for OrderBookV6 MUST match the
    /// deployed runtime bytecode.
    function testRuntimeCodeOrderBook() external {
        OrderBookV6 ob = new OrderBookV6();
        assertEq(keccak256(ORDERBOOK_RUNTIME_CODE), keccak256(address(ob).code));
    }

    /// The precompiled runtime code constant for OrderBookV6SubParser MUST
    /// match the deployed runtime bytecode.
    function testRuntimeCodeSubParser() external {
        OrderBookV6SubParser subParser = new OrderBookV6SubParser();
        assertEq(keccak256(SUB_PARSER_RUNTIME_CODE), keccak256(address(subParser).code));
    }

    /// The generated deployed address for OrderBookV6 MUST match the deploy
    /// library constant.
    function testGeneratedDeployedAddressOrderBook() external pure {
        assertEq(ORDERBOOK_GENERATED_ADDRESS, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
    }

    /// The generated deployed address for OrderBookV6SubParser MUST match the
    /// deploy library constant.
    function testGeneratedDeployedAddressSubParser() external pure {
        assertEq(SUB_PARSER_GENERATED_ADDRESS, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
    }

    /// After calling etchOrderBook, both contracts MUST have the expected
    /// codehash at their expected addresses.
    function testEtchOrderBook() external {
        LibOrderBookDeploy.etchOrderBook(vm);

        assertEq(
            LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH
        );
        assertEq(
            LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );
    }

    /// Calling etchOrderBook twice MUST be idempotent — codehashes remain
    /// correct on the second call.
    function testEtchOrderBookIdempotent() external {
        LibOrderBookDeploy.etchOrderBook(vm);
        LibOrderBookDeploy.etchOrderBook(vm);

        assertEq(
            LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH
        );
        assertEq(
            LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );
    }
}
