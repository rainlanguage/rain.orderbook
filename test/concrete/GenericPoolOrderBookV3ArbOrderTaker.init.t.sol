// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {GenericPoolOrderBookV3ArbOrderTakerTest} from "test/util/abstract/GenericPoolOrderBookV3ArbOrderTakerTest.sol";
import {Clones} from "openzeppelin-contracts/contracts/proxy/Clones.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";
import {OrderBookV3ArbOrderTakerConfigV1} from "src/concrete/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {EvaluableConfigV3, IExpressionDeployerV3} from "src/interface/unstable/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter/interface/IParserV1.sol";
import {NonZeroBeforeArbInputs} from "src/abstract/OrderBookV3ArbOrderTaker.sol";

contract GenericPoolOrderBookV3ArbOrderTakerInitTest is GenericPoolOrderBookV3ArbOrderTakerTest {
    function testGenericPoolOrderBookV3ArbOrderTakerInitInvalidInputs(bytes memory io) public {
        vm.assume(io.length >= 2);
        vm.assume(io.length % 2 == 0);
        vm.assume(io[0] != 0);
        address testArb = Clones.clone(iImplementation);
        vm.mockCall(
            iDeployer,
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(address(0), address(0), address(0), io)
        );
        vm.expectRevert(abi.encodeWithSelector(NonZeroBeforeArbInputs.selector, uint256(uint8(io[0]))));
        ICloneableV2(testArb).initialize(
            abi.encode(
                OrderBookV3ArbOrderTakerConfigV1(
                    address(0),
                    EvaluableConfigV3(
                        IExpressionDeployerV3(iDeployer),
                        // Need a nonzero source count to enter the code path
                        // that deploys the "before arb" expression.
                        hex"01",
                        new uint256[](0)
                    ),
                    ""
                )
            )
        );
    }
}
