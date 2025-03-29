// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {IOrderBookV4, Quote} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {
    OrderConfigV4,
    EvaluableV4,
    TaskV2,
    OrderV3,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {TokenSelfTrade} from "src/concrete/ob/OrderBook.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

/// @title OrderBookQuoteTest
contract OrderBookQuoteTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    /// Dead orders always eval to false.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteDeadOrder(Quote memory quoteConfig) external view {
        (bool success, uint256 maxOutput, uint256 ioRatio) = iOrderbook.quote(quoteConfig);
        assert(!success);
        assertEq(maxOutput, 0);
        assertEq(ioRatio, 0);
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes[] memory rainlang,
        uint256 depositAmount,
        uint256[] memory expectedMaxOutput,
        uint256[] memory expectedIoRatio
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        config.validOutputs[0].token = address(iToken0);
        config.validOutputs[0].decimals = 12;
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(12));

        config.validInputs[0].token = address(iToken1);
        config.validInputs[0].decimals = 6;
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));

        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        vm.prank(owner);
        iOrderbook.deposit2(
            config.validOutputs[0].token, config.validOutputs[0].vaultId, depositAmount, new TaskV2[](0)
        );

        for (uint256 i = 0; i < rainlang.length; i++) {
            config.evaluable.bytecode = iParserV2.parse2(rainlang[i]);
            vm.prank(owner);
            iOrderbook.addOrder2(config, new TaskV2[](0));

            OrderV3 memory order = OrderV3({
                owner: owner,
                evaluable: config.evaluable,
                validInputs: config.validInputs,
                validOutputs: config.validOutputs,
                nonce: config.nonce
            });

            Quote memory quoteConfig =
                Quote({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
            (bool success, uint256 maxOutput, uint256 ioRatio) = iOrderbook.quote(quoteConfig);
            assert(success);
            assertEq(maxOutput, expectedMaxOutput[i], "max output");
            assertEq(ioRatio, expectedIoRatio[i], "io ratio");
        }
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes memory rainlang,
        uint256 depositAmount,
        uint256 expectedMaxOutput,
        uint256 expectedIoRatio
    ) internal {
        bytes[] memory rainlangArray = new bytes[](1);
        rainlangArray[0] = rainlang;

        uint256[] memory expectedMaxOutputArray = new uint256[](1);
        expectedMaxOutputArray[0] = expectedMaxOutput;

        uint256[] memory expectedIoRatioArray = new uint256[](1);
        expectedIoRatioArray[0] = expectedIoRatio;

        checkQuote(owner, config, rainlangArray, depositAmount, expectedMaxOutputArray, expectedIoRatioArray);
    }

    /// forge-config: default.fuzz.runs = 100
    function testQuoteSimple(address owner, OrderConfigV4 memory config, uint256 depositAmount) external {
        depositAmount = bound(depositAmount, 1e18, type(uint256).max / 1e6);
        checkQuote(owner, config, "_ _:1 2;", depositAmount, 1e18, 2e18);
    }

    /// The output will be maxed at the deposit in the vault.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteMaxOutput(address owner, OrderConfigV4 memory config, uint256 depositAmount) external {
        depositAmount = bound(depositAmount, 1, 1e12);
        checkQuote(owner, config, "_ _:1 2;:;", depositAmount, depositAmount * 1e6, 2e18);
    }

    /// Can access context.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteContextSender(address owner, OrderConfigV4 memory config, uint256 depositAmount) external {
        // Max amount needs to be small enough to be scaled up to 18 decimals
        // from 12 decimals.
        depositAmount = bound(depositAmount, 1e18, type(uint256).max / 1e6);

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes[] memory rainlang = new bytes[](10);
        // quote msg.sender
        rainlang[0] = "_ _:1 context<0 0>();:;";
        // orderbook
        rainlang[1] = "_ _:1 context<0 1>();:;";
        // We can't easily check order hash with this setup
        // rainlang[2] = "_ _:1 context<1 0>();:;";
        // quote order owner
        rainlang[2] = "_ _:1 context<1 1>();:;";
        // quote order counterparty (will be quoter)
        rainlang[3] = "_ _:1 context<1 2>();:;";
        // calculations won't be in context for quote
        // inputs
        // vault input token
        rainlang[4] = "_ _:1 context<3 0>();:;";
        // vault input token decimals
        rainlang[5] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(input-token-decimals() context<3 1>()) \"input decimals\"),",
                ":ensure(equal-to(input-token-decimals() 6) \"input decimals literal\"),",
                "_ _:1 context<3 1>();:;"
            )
        );
        // vault io vault id
        // not easy to test with this setup
        // rainlang[6] = "_ _:1 context<3 2>();:;";
        // input vault balance before
        rainlang[6] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(input-vault-before() context<3 3>()) \"input vault before\"),",
                ":ensure(equal-to(input-vault-before() 0) \"input vault before literal\"),",
                "_ _:1 context<3 3>();:;"
            )
        );
        // outputs
        // vault output token
        rainlang[7] = "_ _:1 context<4 0>();:;";
        // vault output token decimals
        rainlang[8] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(output-token-decimals() context<4 1>()) \"output decimals\"),",
                ":ensure(equal-to(output-token-decimals() 12) \"output decimals literal\"),",
                "_ _:1 context<4 1>();:;"
            )
        );
        // vault io vault id
        // not easy to test with this setup
        // rainlang[9] = "_ _:1 context<4 2>();:;";
        // output vault balance before
        rainlang[9] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(output-vault-before() context<4 3>()) \"output vault before\"),",
                ":ensure(equal-to(output-vault-before() ",
                depositAmount.toString(),
                "e-12) \"output vault before literal\"),",
                "_ _:1 context<4 3>();:;"
            )
        );

        uint256[] memory expectedMaxOutput = new uint256[](10);
        expectedMaxOutput[0] = 1e18;
        expectedMaxOutput[1] = 1e18;
        expectedMaxOutput[2] = 1e18;
        expectedMaxOutput[3] = 1e18;
        expectedMaxOutput[4] = 1e18;
        expectedMaxOutput[5] = 1e18;
        expectedMaxOutput[6] = 1e18;
        expectedMaxOutput[7] = 1e18;
        expectedMaxOutput[8] = 1e18;
        expectedMaxOutput[9] = 1e18;

        uint256[] memory expectedIoRatio = new uint256[](10);
        expectedIoRatio[0] = uint256(uint160(address(this)));
        expectedIoRatio[1] = uint256(uint160(address(iOrderbook)));
        expectedIoRatio[2] = uint256(uint160(owner));
        expectedIoRatio[3] = uint256(uint160(address(this)));
        expectedIoRatio[4] = uint256(uint160(address(iToken1)));
        // Input decimals scaled to 18 fixed point value.
        expectedIoRatio[5] = 6e18;
        expectedIoRatio[6] = 0;
        expectedIoRatio[7] = uint256(uint160(address(iToken0)));
        expectedIoRatio[8] = 12e18;
        // Output decimals scaled to 18 fixed point value from 12.
        expectedIoRatio[9] = depositAmount * 1e6;

        checkQuote(owner, config, rainlang, depositAmount, expectedMaxOutput, expectedIoRatio);
    }
}
