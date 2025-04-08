// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    IOrderBookV5,
    QuoteV2,
    OrderConfigV4,
    EvaluableV4,
    TaskV2,
    OrderV4,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {TokenSelfTrade} from "src/concrete/ob/OrderBook.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

/// @title OrderBookQuoteTest
contract OrderBookQuoteTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    using LibDecimalFloat for Float;

    /// Dead orders always eval to false.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteDeadOrder(QuoteV2 memory quoteConfig) external view {
        (bool success, Float memory maxOutput, Float memory ioRatio) = iOrderbook.quote2(quoteConfig);
        assert(!success);
        assertTrue(maxOutput.eq(Float(0, 0)), "max output");
        assertTrue(ioRatio.eq(Float(0, 0)), "io ratio");
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes[] memory rainlang,
        Float memory depositAmount,
        Float[] memory expectedMaxOutput,
        Float[] memory expectedIoRatio
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        uint256 depositAmount18 = depositAmount.toFixedDecimalLossless(18);

        config.validOutputs[0].token = address(iToken0);
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(12));

        config.validInputs[0].token = address(iToken1);
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));

        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iOrderbook), depositAmount18),
            abi.encode(true)
        );
        vm.prank(owner);
        iOrderbook.deposit3(
            config.validOutputs[0].token, config.validOutputs[0].vaultId, depositAmount, new TaskV2[](0)
        );

        for (uint256 i = 0; i < rainlang.length; i++) {
            config.evaluable.bytecode = iParserV2.parse2(rainlang[i]);
            vm.prank(owner);
            iOrderbook.addOrder3(config, new TaskV2[](0));

            OrderV4 memory order = OrderV4({
                owner: owner,
                evaluable: config.evaluable,
                validInputs: config.validInputs,
                validOutputs: config.validOutputs,
                nonce: config.nonce
            });

            QuoteV2 memory quoteConfig =
                QuoteV2({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
            (bool success, Float memory maxOutput, Float memory ioRatio) = iOrderbook.quote2(quoteConfig);
            assert(success);
            assertTrue(maxOutput.eq(expectedMaxOutput[i]), "max output");
            assertTrue(ioRatio.eq(expectedIoRatio[i]), "io ratio");
        }
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes memory rainlang,
        Float memory depositAmount,
        Float memory expectedMaxOutput,
        Float memory expectedIoRatio
    ) internal {
        bytes[] memory rainlangArray = new bytes[](1);
        rainlangArray[0] = rainlang;

        Float[] memory expectedMaxOutputArray = new Float[](1);
        expectedMaxOutputArray[0] = expectedMaxOutput;

        Float[] memory expectedIoRatioArray = new Float[](1);
        expectedIoRatioArray[0] = expectedIoRatio;

        checkQuote(owner, config, rainlangArray, depositAmount, expectedMaxOutputArray, expectedIoRatioArray);
    }

    /// forge-config: default.fuzz.runs = 100
    function testQuoteSimple(address owner, OrderConfigV4 memory config, uint256 depositAmount18) external {
        depositAmount18 = bound(depositAmount18, 1e18, type(uint256).max / 1e6);
        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);
        checkQuote(owner, config, "_ _:1 2;", depositAmount, Float(1, 0), Float(2, 0));
    }

    /// The output will be maxed at the deposit in the vault.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteMaxOutput(address owner, OrderConfigV4 memory config, uint256 depositAmount18) external {
        depositAmount18 = bound(depositAmount18, 1, 1e12);
        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 12);
        checkQuote(owner, config, "_ _:1 2;:;", depositAmount, depositAmount.multiply(Float(1, 6)), Float(2, 0));
    }

    /// Can access context.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteContextSender(address owner, OrderConfigV4 memory config, uint256 depositAmount18) external {
        // Max amount needs to be small enough to be scaled up to 18 decimals
        // from 12 decimals.
        depositAmount18 = bound(depositAmount18, 1e18, type(uint256).max / 1e6);

        Float memory depositAmount = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18, 18);

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
                depositAmount18.toString(),
                "e-12) \"output vault before literal\"),",
                "_ _:1 context<4 3>();:;"
            )
        );

        Float[] memory expectedMaxOutput = new Float[](10);
        expectedMaxOutput[0] = Float(1, 0);
        expectedMaxOutput[1] = Float(1, 0);
        expectedMaxOutput[2] = Float(1, 0);
        expectedMaxOutput[3] = Float(1, 0);
        expectedMaxOutput[4] = Float(1, 0);
        expectedMaxOutput[5] = Float(1, 0);
        expectedMaxOutput[6] = Float(1, 0);
        expectedMaxOutput[7] = Float(1, 0);
        expectedMaxOutput[8] = Float(1, 0);
        expectedMaxOutput[9] = Float(1, 0);

        Float[] memory expectedIoRatio = new Float[](10);
        expectedIoRatio[0] = LibDecimalFloat.fromFixedDecimalLosslessMem(uint256(uint160(address(this))), 18);
        expectedIoRatio[1] = LibDecimalFloat.fromFixedDecimalLosslessMem(uint256(uint160(address(iOrderbook))), 18);
        expectedIoRatio[2] = LibDecimalFloat.fromFixedDecimalLosslessMem(uint256(uint160(owner)), 18);
        expectedIoRatio[3] = LibDecimalFloat.fromFixedDecimalLosslessMem(uint256(uint160(address(this))), 18);
        expectedIoRatio[4] = LibDecimalFloat.fromFixedDecimalLosslessMem(uint256(uint160(address(iToken1))), 18);
        // Input decimals scaled to 18 fixed point value.
        expectedIoRatio[5] = LibDecimalFloat.fromFixedDecimalLosslessMem(6e18, 18);
        expectedIoRatio[6] = LibDecimalFloat.fromFixedDecimalLosslessMem(0, 18);
        expectedIoRatio[7] = LibDecimalFloat.fromFixedDecimalLosslessMem(uint256(uint160(address(iToken0))), 18);
        expectedIoRatio[8] = LibDecimalFloat.fromFixedDecimalLosslessMem(12e18, 18);
        // Output decimals scaled to 18 fixed point value from 12.
        expectedIoRatio[9] = LibDecimalFloat.fromFixedDecimalLosslessMem(depositAmount18 * 1e6, 18);

        checkQuote(owner, config, rainlang, depositAmount, expectedMaxOutput, expectedIoRatio);
    }
}
