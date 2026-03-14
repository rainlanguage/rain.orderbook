// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {
    RouteProcessorOrderBookV6ArbOrderTaker,
    OrderBookV6ArbConfig
} from "src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
import {EvaluableV4, SignedContextV1, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";

contract RouteProcessorOrderBookV6ArbOrderTakerInvalidConstructorTest is Test {
    /// Constructor MUST revert when implementationData is empty (cannot
    /// abi.decode an address from zero bytes).
    function testConstructorRevertsEmptyImplementationData() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        // abi.decode on too-short data produces a bare revert with no data.
        vm.expectRevert(bytes(""));
        new RouteProcessorOrderBookV6ArbOrderTaker(
            OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                }),
                ""
            )
        );
    }

    /// Constructor MUST revert when implementationData is malformed (too
    /// short to contain an ABI-encoded address).
    function testConstructorRevertsMalformedImplementationData() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        // abi.decode on too-short data produces a bare revert with no data.
        vm.expectRevert(bytes(""));
        new RouteProcessorOrderBookV6ArbOrderTaker(
            OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                }),
                hex"deadbeef"
            )
        );
    }
}
