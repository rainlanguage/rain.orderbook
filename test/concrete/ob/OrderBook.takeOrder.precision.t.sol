// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Test.sol";
import {OrderBookExternalRealTest, IERC20} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    OrderV4,
    TakeOrdersConfigV4,
    TakeOrderConfigV4,
    IOV2,
    OrderConfigV4,
    EvaluableV4,
    SignedContextV1,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {console2} from "forge-std/Test.sol";

/// @title OrderBookTakeOrderPrecisionTest
/// @notice A test harness for testing the OrderBook takeOrder function.
contract OrderBookTakeOrderPrecisionTest is OrderBookExternalRealTest {
    using LibDecimalFloat for Float;

    function checkPrecision(
        bytes memory rainString,
        uint8 outputTokenDecimals,
        uint8 inputTokenDecimals,
        Float memory expectedTakerTotalInput,
        Float memory expectedTakerTotalOutput
    ) internal {
        bytes32 vaultId = 0;
        address inputToken = address(0x100);
        address outputToken = address(0x101);

        // Etch with invalid.
        vm.etch(outputToken, hex"fe");
        vm.etch(inputToken, hex"fe");

        vm.mockCall(
            inputToken, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(inputTokenDecimals)
        );
        vm.mockCall(
            outputToken, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(outputTokenDecimals)
        );

        {
            (uint256 absoluteDepositAmount, bool lossless) =
                expectedTakerTotalInput.toFixedDecimalLossy(outputTokenDecimals);
            if (!lossless) {
                ++absoluteDepositAmount;
            }
            vm.mockCall(
                outputToken,
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector, address(this), address(iOrderbook), absoluteDepositAmount
                ),
                abi.encode(true)
            );
        }

        {
            (uint256 absoluteTakerInputAmount, bool lossless) =
                expectedTakerTotalInput.toFixedDecimalLossy(outputTokenDecimals);
            (lossless);
            vm.mockCall(
                outputToken,
                abi.encodeWithSelector(IERC20.transfer.selector, address(this), absoluteTakerInputAmount),
                abi.encode(true)
            );
        }

        {
            (uint256 absoluteTakerOutputAmount, bool lossless) =
                expectedTakerTotalOutput.toFixedDecimalLossy(inputTokenDecimals);
            if (!lossless) {
                ++absoluteTakerOutputAmount;
            }
            vm.mockCall(
                inputToken,
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector, address(this), address(iOrderbook), absoluteTakerOutputAmount
                ),
                abi.encode(true)
            );
        }

        OrderConfigV4 memory config;
        {
            IOV2[] memory validInputs = new IOV2[](1);
            validInputs[0] = IOV2(inputToken, vaultId);
            IOV2[] memory validOutputs = new IOV2[](1);
            validOutputs[0] = IOV2(outputToken, vaultId);
            // These numbers are known to cause large rounding errors if the
            // precision is not handled correctly.
            bytes memory bytecode = iParserV2.parse2(rainString);
            EvaluableV4 memory evaluable = EvaluableV4(iInterpreter, iStore, bytecode);
            config = OrderConfigV4(evaluable, validInputs, validOutputs, bytes32(0), bytes32(0), "");
        }

        {
            if (expectedTakerTotalInput.gt(Float(0, 0))) {
                iOrderbook.deposit3(outputToken, vaultId, expectedTakerTotalInput, new TaskV2[](0));
            }
            assertTrue(iOrderbook.vaultBalance2(address(this), outputToken, vaultId).eq(expectedTakerTotalInput));
            vm.recordLogs();
            iOrderbook.addOrder3(config, new TaskV2[](0));
            Vm.Log[] memory entries = vm.getRecordedLogs();
            assertEq(entries.length, 1);
            (,, OrderV4 memory order) = abi.decode(entries[0].data, (address, bytes32, OrderV4));

            TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
            orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));
            TakeOrdersConfigV4 memory takeOrdersConfig =
                TakeOrdersConfigV4(Float(0, 0), Float(type(int256).max, 0), Float(type(int256).max, 0), orders, "");
            (Float memory totalTakerInput, Float memory totalTakerOutput) = iOrderbook.takeOrders3(takeOrdersConfig);
            assertTrue(totalTakerInput.eq(expectedTakerTotalInput), "input");
            assertTrue(totalTakerOutput.eq(expectedTakerTotalOutput), "output");
        }

        assertTrue(iOrderbook.vaultBalance2(address(this), outputToken, vaultId).eq(Float(0, 0)), "vault balance");
    }

    // Older versions of OB had precision issues with this IO setup.
    // bytes memory knownBad = "output-max io-ratio:157116.36568049186712991 0.000318235466963885;:;";
    bytes constant KNOWN_BAD = bytes("output-max io-ratio:157116.36568049186712991 0.000318235466963885;:;");

    function testTakeOrderPrecisionKnownBad01() public {
        checkPrecision(
            KNOWN_BAD, 18, 18, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad02() public {
        checkPrecision(
            KNOWN_BAD, 18, 6, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad03() public {
        checkPrecision(
            KNOWN_BAD, 19, 6, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad04() public {
        checkPrecision(
            KNOWN_BAD, 20, 6, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad05() public {
        checkPrecision(
            KNOWN_BAD, 21, 6, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad06() public {
        checkPrecision(
            KNOWN_BAD, 50, 6, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad07() public {
        checkPrecision(
            KNOWN_BAD, 6, 18, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad08() public {
        checkPrecision(
            KNOWN_BAD, 6, 19, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad09() public {
        checkPrecision(
            KNOWN_BAD, 6, 20, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad10() public {
        checkPrecision(
            KNOWN_BAD, 6, 21, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }

    function testTakeOrderPrecisionKnownBad11() public {
        checkPrecision(
            KNOWN_BAD, 6, 50, Float(157116365680491867129910, -18), Float(4999999999999984457923789473657330035, -35)
        );
    }
}
