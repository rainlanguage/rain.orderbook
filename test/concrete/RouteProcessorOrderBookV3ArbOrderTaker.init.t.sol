// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {RouteProcessorOrderBookV3ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV3ArbOrderTakerTest.sol";
import {Clones} from "openzeppelin-contracts/contracts/proxy/Clones.sol";
import {ICloneableV2} from "rain.factory/src/interface/ICloneableV2.sol";
import {OrderBookV3ArbOrderTakerConfigV1} from "src/concrete/RouteProcessorOrderBookV3ArbOrderTaker.sol";
import {EvaluableConfigV3, IExpressionDeployerV3} from "src/interface/unstable/IOrderBookV3.sol";
import {NonZeroBeforeArbInputs} from "src/abstract/OrderBookV3ArbOrderTaker.sol";

contract RouteProcessorOrderBookV3ArbOrderTakerInitTest is RouteProcessorOrderBookV3ArbOrderTakerTest {
    function testRouteProcessorOrderBookV3ArbOrderTakerInitInvalidInputs() public {
        address testArb = Clones.clone(iImplementation);
        vm.mockCall(
            iDeployer,
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            abi.encode(address(0), address(0), address(0), "0100")
        );
        vm.expectRevert(abi.encodeWithSelector(NonZeroBeforeArbInputs.selector, 48));
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
                    abi.encode(iRefundoor)
                )
            )
        );
    }
}
