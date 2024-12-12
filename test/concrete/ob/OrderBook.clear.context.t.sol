// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 thedavidmeister
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV3,
    OrderV3,
    TaskV1,
    ClearConfig,
    SignedContextV1,
    IO,
    EvaluableV3
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

contract OrderBookClearOrderContextTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    /// forge-config: default.fuzz.runs = 10
    function testContextEmptyStack(
        address alice,
        address bob,
        uint256 aliceInputVaultId,
        uint256 aliceOutputVaultId,
        uint256 bobInputVaultId,
        uint256 bobOutputVaultId
    ) external {
        vm.assume(alice != bob);

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes memory rainStringAlice = bytes(
            string.concat(
                usingWordsFrom,
                "_ _:5 2;",
                ":ensure(equal-to(input-token() ",
                address(iToken0).toHexString(),
                ") \"input token\"),",
                ":ensure(equal-to(input-token-decimals() 12) \"input token decimals\"),",
                ":ensure(equal-to(input-vault-id() ",
                aliceInputVaultId.toHexString(),
                ") \"input token vault\"),",
                ":ensure(equal-to(input-vault-before() 0) \"input vault before\"),",
                ":ensure(equal-to(output-token() ",
                address(iToken1).toHexString(),
                ") \"output token\"),",
                ":ensure(equal-to(output-token-decimals() 6) \"output token decimals\"),",
                ":ensure(equal-to(output-vault-id() ",
                aliceOutputVaultId.toHexString(),
                ") \"output token vault\"),",
                ":ensure(equal-to(output-vault-before() 100) \"output vault before\"),",
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
                ") \"Bob\"),",
                ":ensure(equal-to(calculated-io-ratio() 2) \"alice io ratio\"),",
                ":ensure(equal-to(output-vault-decrease() 1.5) \"alice output vault decrease\"),",
                ":ensure(equal-to(input-vault-increase() 3) \"alice input vault increase\"),",
                ":ensure(equal-to(calculated-max-output() 5) \"alice max output\");"
            )
        );

        bytes memory rainStringBob = bytes(
            string.concat(
                string.concat(
                    usingWordsFrom,
                    "_ _:3 0.5;",
                    ":ensure(equal-to(input-token() ",
                    address(iToken1).toHexString(),
                    ") \"input token\"),",
                    ":ensure(equal-to(input-token-decimals() 6) \"input token decimals\"),",
                    ":ensure(equal-to(input-vault-id() ",
                    bobInputVaultId.toHexString(),
                    ") \"input token vault\"),",
                    ":ensure(equal-to(input-vault-before() 0) \"input vault before\"),",
                    ":ensure(equal-to(output-token() ",
                    address(iToken0).toHexString(),
                    ") \"output token\"),",
                    ":ensure(equal-to(output-token-decimals() 12) \"output token decimals\"),",
                    ":ensure(equal-to(output-vault-id() ",
                    bobOutputVaultId.toHexString(),
                    ") \"output token vault\"),",
                    ":ensure(equal-to(output-vault-before() 100) \"output vault before\"),",
                    ":ensure(equal-to(orderbook() ",
                    address(iOrderbook).toHexString(),
                    ") \"OrderBook\"),"
                ),
                ":ensure(equal-to(order-clearer() ",
                address(this).toHexString(),
                ") \"clearer\"),",
                ":ensure(equal-to(order-owner() ",
                bob.toHexString(),
                ") \"Bob\"),",
                ":ensure(equal-to(order-counterparty() ",
                alice.toHexString(),
                ") \"Alice\"),",
                ":ensure(equal-to(calculated-io-ratio() 0.5) \"bob io ratio\"),",
                ":ensure(equal-to(output-vault-decrease() 3) \"bob output vault decrease\"),",
                ":ensure(equal-to(input-vault-increase() 1.5) \"bob input vault increase\"),",
                ":ensure(equal-to(calculated-max-output() 3) \"bob max output\");"
            )
        );

        OrderConfigV3 memory configAlice;
        {
            IO[] memory validInputsAlice = new IO[](1);
            validInputsAlice[0] = IO({token: address(iToken0), decimals: 12, vaultId: aliceInputVaultId});
            IO[] memory validOutputsAlice = new IO[](1);
            validOutputsAlice[0] = IO({token: address(iToken1), decimals: 6, vaultId: aliceOutputVaultId});
            configAlice = OrderConfigV3({
                evaluable: EvaluableV3({
                    bytecode: iParserV2.parse2(rainStringAlice),
                    interpreter: iInterpreter,
                    store: iStore
                }),
                validInputs: validInputsAlice,
                validOutputs: validOutputsAlice,
                nonce: 0,
                secret: 0,
                meta: ""
            });
        }
        OrderConfigV3 memory configBob;
        {
            IO[] memory validInputsBob = new IO[](1);
            validInputsBob[0] = IO({token: address(iToken1), decimals: 6, vaultId: bobInputVaultId});

            IO[] memory validOutputsBob = new IO[](1);
            validOutputsBob[0] = IO({token: address(iToken0), decimals: 12, vaultId: bobOutputVaultId});

            configBob = OrderConfigV3({
                evaluable: EvaluableV3({bytecode: iParserV2.parse2(rainStringBob), interpreter: iInterpreter, store: iStore}),
                validInputs: validInputsBob,
                validOutputs: validOutputsBob,
                nonce: 0,
                secret: 0,
                meta: ""
            });
        }

        vm.mockCall(
            configAlice.validOutputs[0].token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook)),
            abi.encode(true)
        );
        vm.mockCall(
            configBob.validOutputs[0].token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook)),
            abi.encode(true)
        );

        OrderV3 memory orderAlice =
            OrderV3(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV3 memory orderBob =
            OrderV3(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder2(configAlice, new TaskV1[](0));

        vm.prank(alice);
        iOrderbook.deposit2(
            configAlice.validOutputs[0].token, configAlice.validOutputs[0].vaultId, 100e6, new TaskV1[](0)
        );

        vm.prank(bob);
        iOrderbook.addOrder2(configBob, new TaskV1[](0));

        vm.prank(bob);
        iOrderbook.deposit2(configBob.validOutputs[0].token, configBob.validOutputs[0].vaultId, 100e12, new TaskV1[](0));

        iOrderbook.clear2(
            orderAlice, orderBob, ClearConfig(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
