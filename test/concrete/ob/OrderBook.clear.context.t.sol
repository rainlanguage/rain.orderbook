// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV4,
    OrderV4,
    TaskV2,
    ClearConfigV2,
    SignedContextV1,
    IOV2,
    EvaluableV4
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";

contract OrderBookClearOrderContextTest is OrderBookExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    /// forge-config: default.fuzz.runs = 10
    function testContextEmptyStack(
        address alice,
        address bob,
        bytes32 aliceInputVaultId,
        bytes32 aliceOutputVaultId,
        bytes32 bobInputVaultId,
        bytes32 bobOutputVaultId
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
                uint256(aliceInputVaultId).toHexString(),
                ") \"input token vault\"),",
                ":ensure(equal-to(input-vault-before() 0) \"input vault before\"),",
                ":ensure(equal-to(output-token() ",
                address(iToken1).toHexString(),
                ") \"output token\"),",
                ":ensure(equal-to(output-token-decimals() 6) \"output token decimals\"),",
                ":ensure(equal-to(output-vault-id() ",
                uint256(aliceOutputVaultId).toHexString(),
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
                    uint256(bobInputVaultId).toHexString(),
                    ") \"input token vault\"),",
                    ":ensure(equal-to(input-vault-before() 0) \"input vault before\"),",
                    ":ensure(equal-to(output-token() ",
                    address(iToken0).toHexString(),
                    ") \"output token\"),",
                    ":ensure(equal-to(output-token-decimals() 12) \"output token decimals\"),",
                    ":ensure(equal-to(output-vault-id() ",
                    uint256(bobOutputVaultId).toHexString(),
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

        OrderConfigV4 memory configAlice;
        {
            IOV2[] memory validInputsAlice = new IOV2[](1);
            validInputsAlice[0] = IOV2({token: address(iToken0), vaultId: aliceInputVaultId});
            IOV2[] memory validOutputsAlice = new IOV2[](1);
            validOutputsAlice[0] = IOV2({token: address(iToken1), vaultId: aliceOutputVaultId});
            configAlice = OrderConfigV4({
                evaluable: EvaluableV4({
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
        OrderConfigV4 memory configBob;
        {
            IOV2[] memory validInputsBob = new IOV2[](1);
            validInputsBob[0] = IOV2({token: address(iToken1), vaultId: bobInputVaultId});

            IOV2[] memory validOutputsBob = new IOV2[](1);
            validOutputsBob[0] = IOV2({token: address(iToken0), vaultId: bobOutputVaultId});

            configBob = OrderConfigV4({
                evaluable: EvaluableV4({bytecode: iParserV2.parse2(rainStringBob), interpreter: iInterpreter, store: iStore}),
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

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV4 memory orderBob =
            OrderV4(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder3(configAlice, new TaskV2[](0));

        vm.prank(alice);
        iOrderbook.deposit3(
            configAlice.validOutputs[0].token, configAlice.validOutputs[0].vaultId, Float(100e6, -18), new TaskV2[](0)
        );

        vm.prank(bob);
        iOrderbook.addOrder3(configBob, new TaskV2[](0));

        vm.prank(bob);
        iOrderbook.deposit3(
            configBob.validOutputs[0].token, configBob.validOutputs[0].vaultId, Float(100e12, -18), new TaskV2[](0)
        );

        iOrderbook.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
