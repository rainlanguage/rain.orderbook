// SPDX-License-Identifier: CAL
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV3,
    OrderV3,
    ActionV1,
    ClearConfig,
    SignedContextV1
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

contract OrderBookClearOrderContextTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    function testContextEmptyStack(
        address alice,
        address bob,
        OrderConfigV3 memory configAlice,
        OrderConfigV3 memory configBob
    ) external {
        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);

        configAlice.validInputs[0].decimals = 12;
        configBob.validInputs[0].decimals = 6;

        configBob.validOutputs[0] = configAlice.validInputs[0];
        configAlice.validOutputs[0] = configBob.validInputs[0];

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes memory rainStringAlice = bytes(
            string.concat(
                usingWordsFrom,
                "_ _:1 1;",
                ":ensure(equal-to(input-token() ",
                address(configAlice.validInputs[0].token).toHexString(),
                ") \"input token\"),",
                ":ensure(equal-to(input-token-decimals() 12) \"input token decimals\"),",
                ":ensure(equal-to(input-vault-id() ",
                configAlice.validInputs[0].vaultId.toHexString(),
                ") \"input token vault\"),",
                ":ensure(equal-to(output-token() ",
                address(configAlice.validOutputs[0].token).toHexString(),
                ") \"output token\"),",
                ":ensure(equal-to(output-token-decimals() 6) \"output token decimals\"),",
                ":ensure(equal-to(output-vault-id() ",
                configAlice.validOutputs[0].vaultId.toHexString(),
                ") \"output token vault\"),",
                ":ensure(equal-to(orderbook() ",
                address(iOrderbook).toHexString(),
                ") \"OrderBook\"),",
                ":ensure(equal-to(order-clearer() ",
                address(this).toHexString(),
                ") \"clearer\"),",
                ":ensure(equal-to(order-owner() ",
                alice.toHexString(),
                ") \"Alice\"),",
                ":ensure(equal-to(order-counterparty() ",
                bob.toHexString(),
                ") \"Bob\");"
            )
        );

        bytes memory rainStringBob = bytes(
            string.concat(
                usingWordsFrom,
                "_ _:1 1;",
                ":ensure(equal-to(input-token() ",
                address(configBob.validInputs[0].token).toHexString(),
                ") \"input token\"),",
                ":ensure(equal-to(input-token-decimals() 6) \"input token decimals\"),",
                ":ensure(equal-to(input-vault-id() ",
                configBob.validInputs[0].vaultId.toHexString(),
                ") \"input token vault\"),",
                ":ensure(equal-to(output-token() ",
                address(configBob.validOutputs[0].token).toHexString(),
                ") \"output token\"),",
                ":ensure(equal-to(output-token-decimals() 12) \"output token decimals\"),",
                ":ensure(equal-to(output-vault-id() ",
                configBob.validOutputs[0].vaultId.toHexString(),
                ") \"output token vault\"),",
                ":ensure(equal-to(orderbook() ",
                address(iOrderbook).toHexString(),
                ") \"OrderBook\"),",
                ":ensure(equal-to(order-clearer() ",
                address(this).toHexString(),
                ") \"clearer\"),",
                ":ensure(equal-to(order-owner() ",
                bob.toHexString(),
                ") \"Bob\"),",
                ":ensure(equal-to(order-counterparty() ",
                alice.toHexString(),
                ") \"Alice\");"
            )
        );

        vm.assume(alice != bob);

        configAlice.evaluable.bytecode = iParserV2.parse2(rainStringAlice);
        configBob.evaluable.bytecode = iParserV2.parse2(rainStringBob);

        OrderV3 memory orderAlice =
            OrderV3(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV3 memory orderBob =
            OrderV3(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder2(configAlice, new ActionV1[](0));

        vm.prank(bob);
        iOrderbook.addOrder2(configBob, new ActionV1[](0));

        iOrderbook.clear2(
            orderAlice, orderBob, ClearConfig(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
