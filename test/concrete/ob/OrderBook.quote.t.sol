// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {IOrderBookV4, Quote} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {
    OrderConfigV3,
    EvaluableV3,
    ActionV1,
    OrderV3,
    SignedContextV1
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";

/// @title OrderBookQuoteTest
contract OrderBookQuoteTest is OrderBookExternalRealTest {
    /// Dead orders always eval to false.
    function testQuoteDeadOrder(Quote memory quoteConfig) external {
        (bool success, uint256 maxOutput, uint256 ioRatio) = iOrderbook.quote(quoteConfig);
        assert(!success);
        assertEq(maxOutput, 0);
        assertEq(ioRatio, 0);
    }

    function checkQuote(
        address owner,
        OrderConfigV3 memory config,
        bytes[] memory rainlang,
        uint256 depositAmount,
        uint256[] memory expectedMaxOutput,
        uint256[] memory expectedIoRatio
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        config.validOutputs[0].token = address(iToken0);
        config.validOutputs[0].decimals = 18;
        config.validInputs[0].token = address(iToken1);
        config.validInputs[0].decimals = 18;

        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        vm.prank(owner);
        iOrderbook.deposit2(
            config.validOutputs[0].token, config.validOutputs[0].vaultId, depositAmount, new ActionV1[](0)
        );

        for (uint256 i = 0; i < rainlang.length; i++) {
            config.evaluable.bytecode = iParserV2.parse2(rainlang[i]);
            vm.prank(owner);
            iOrderbook.addOrder2(config, new ActionV1[](0));

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
            assertEq(maxOutput, expectedMaxOutput[i]);
            assertEq(ioRatio, expectedIoRatio[i]);
        }
    }

    function checkQuote(
        address owner,
        OrderConfigV3 memory config,
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

    function testQuoteSimple(address owner, OrderConfigV3 memory config, uint256 depositAmount) external {
        depositAmount = bound(depositAmount, 1e18, type(uint256).max);
        checkQuote(owner, config, "_ _:1 2;", depositAmount, 1e18, 2e18);
    }

    /// The output will be maxed at the deposit in the vault.
    function testQuoteMaxOutput(address owner, OrderConfigV3 memory config, uint256 depositAmount) external {
        depositAmount = bound(depositAmount, 1, 1e18);
        checkQuote(owner, config, "_ _:1 2;:;", depositAmount, depositAmount, 2e18);
    }

    /// Can access context.
    function testQuoteContextSender(address owner, OrderConfigV3 memory config, uint256 depositAmount) external {
        depositAmount = bound(depositAmount, 1e18, type(uint256).max);

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
        // vault io token decimals
        rainlang[5] = "_ _:1 context<3 1>();:;";
        // vault io vault id
        // not easy to test with this setup
        // rainlang[6] = "_ _:1 context<3 2>();:;";
        // vault io balance before
        rainlang[6] = "_ _:1 context<3 3>();:;";
        // outputs
        // vault output token
        rainlang[7] = "_ _:1 context<4 0>();:;";
        // vault io token decimals
        rainlang[8] = "_ _:1 context<4 1>();:;";
        // vault io vault id
        // not easy to test with this setup
        // rainlang[9] = "_ _:1 context<4 2>();:;";
        // vault io balance before
        rainlang[9] = "_ _:1 context<4 3>();:;";

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
        expectedIoRatio[5] = 18;
        expectedIoRatio[6] = 0;
        expectedIoRatio[7] = uint256(uint160(address(iToken0)));
        expectedIoRatio[8] = 18;
        expectedIoRatio[9] = depositAmount;

        checkQuote(owner, config, rainlang, depositAmount, expectedMaxOutput, expectedIoRatio);
    }
}
