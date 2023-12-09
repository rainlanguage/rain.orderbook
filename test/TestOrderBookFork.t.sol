// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test, console2} from "forge-std/Test.sol";
import "src/interface/unstable/IOrderBookV3ArbOrderTaker.sol";
import "lib/rain.interpreter/src/interface/unstable/IExpressionDeployerV3.sol";
import {
    IOrderBookV3,
    OrderV2,
    EvaluableConfigV3,
    OrderConfigV2,
    TakeOrderConfigV2,
    SignedContextV1,
    TakeOrdersConfigV2
} from "src/interface/unstable/IOrderBookV3.sol";

IOrderBookV3ArbOrderTaker constant POLYGON_NHS_ARB_CONTRACT =
    IOrderBookV3ArbOrderTaker(0xD4A7e432b5E56535F0a671144551ef833dc2c569);

IOrderBookV3 constant POLYGON_NHS_ORDERBOOK = IOrderBookV3(0x16d518706d666C549DA7Bd31110623B09eF23AbB);

bytes constant SELL_ROUTE_NHS =
//offset
    hex"0000000000000000000000000000000000000000000000000000000000000020"
    //stream length
    hex"0000000000000000000000000000000000000000000000000000000000000042"
    //command 2 = processUserERC20
    hex"02"
    //token address
    hex"84342e932797fc62814189f01f0fb05f52519708"
    //number of pools
    hex"01"
    // pool share
    hex"ffff"
    // pool type
    hex"00"
    // pool address
    hex"e427b62b495c1dfe1fe9f78bebfceb877ad05dce"
    // direction 1
    hex"01"
    // to
    hex"D4A7e432b5E56535F0a671144551ef833dc2c569"
    // padding
    hex"000000000000000000000000000000000000000000000000000000000000";

contract TestNHSOrderBook is Test {
    string constant FORK_RPC = "https://polygon.llamarpc.com";

    // Choosing a block right before limit orders were removed.
    uint256 constant FORK_BLOCK_NUMBER = 50906935;

    address constant NHS_ORDER_OWNER = address(0xf098172786a87FA7426eA811Ff25D31D599f766D);
    address constant NHS_APPROVED_EOA = address(0xf098172786a87FA7426eA811Ff25D31D599f766D);

    function selectPolygonFork() internal {
        uint256 fork = vm.createFork(FORK_RPC);
        vm.selectFork(fork);
        vm.rollFork(FORK_BLOCK_NUMBER);
    }

    function getVolSellOrder() internal pure returns (OrderV2 memory) {
        /// @dev https://polygonscan.com/tx/0x470130cf72761778fbd0951364cb4c02396fccd514021d6024ab93ba3e1a2fbf
        bytes memory orderData =
            hex"000000000000000000000000f098172786a87fa7426ea811ff25d31d599f766d000000000000000000000000475c3cda27b6dd89148220915b4e4f57b56a0be10000000000000000000000000000000000000000000000000000000000000080e4cda2a5c590002bf8a260ec65b65cb9ca82a78ffd60b1056f665d89e387a3bb000000000000000000000000f098172786a87fa7426ea811ff25d31d599f766d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000005b84e7ca07b6c7d68e472d0ba9db93a783a7504a0000000000000000000000008bb704a1c16e47305170c553a332b0ade22c222f00000000000000000000000080e2838517cf87484447b6ff546d8b752d3e4dcc00000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000c2132d05d31c914a87c6611c10748aeb04b58e8f0000000000000000000000000000000000000000000000000000000000000006d6e7c8f779cce6c489b1deffa0d3024e4e2257d8fb0c45a945ca4b00e838e8f8000000000000000000000000000000000000000000000000000000000000000100000000000000000000000084342e932797fc62814189f01f0fb05f525197080000000000000000000000000000000000000000000000000000000000000012d6e7c8f779cce6c489b1deffa0d3024e4e2257d8fb0c45a945ca4b00e838e8f8";

        (,, OrderV2 memory volSellOrder,) = abi.decode(orderData, (address, IExpressionDeployerV3, OrderV2, bytes32));
        return volSellOrder;
    }

    function getLimitSellOrder() internal pure returns (OrderV2 memory) {
        /// @dev https://polygonscan.com/tx/0xb0575c41e49bdb5275186a41d0231b4c80b4776e78bc20f331971cc5d771d3ec
        bytes memory orderData =
            hex"000000000000000000000000f098172786a87fa7426ea811ff25d31d599f766d000000000000000000000000475c3cda27b6dd89148220915b4e4f57b56a0be10000000000000000000000000000000000000000000000000000000000000080d0c0b8d1870e5e3f6cffa2c945a364037e3441c80192a456b7bc2c8860ad20e8000000000000000000000000f098172786a87fa7426ea811ff25d31d599f766d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000005b84e7ca07b6c7d68e472d0ba9db93a783a7504a0000000000000000000000008bb704a1c16e47305170c553a332b0ade22c222f00000000000000000000000062c8bd5637c4c2369478a1bd78f34a7095d19b7e00000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000c2132d05d31c914a87c6611c10748aeb04b58e8f0000000000000000000000000000000000000000000000000000000000000006b5c2e4ab8e4e8e139181dcd68c106f1601b8d3dc584765da2e8764c28719fe97000000000000000000000000000000000000000000000000000000000000000100000000000000000000000084342e932797fc62814189f01f0fb05f525197080000000000000000000000000000000000000000000000000000000000000012b5c2e4ab8e4e8e139181dcd68c106f1601b8d3dc584765da2e8764c28719fe97";

        (,, OrderV2 memory limitSellOrder,) = abi.decode(orderData, (address, IExpressionDeployerV3, OrderV2, bytes32));
        return limitSellOrder;
    }

    function testNhsLimitVolArb() public {
        selectPolygonFork();
        OrderV2 memory limitSellOrder = getLimitSellOrder();
        OrderV2 memory volSellOrder = getVolSellOrder();

        OrderV2[] memory orders = new OrderV2[](2);
        orders[0] = limitSellOrder;
        orders[1] = volSellOrder;

        takeOrder(orders, SELL_ROUTE_NHS);
    }

    function testOnlyVolArb() public {
        selectPolygonFork();
        OrderV2 memory volSellOrder = getVolSellOrder();

        OrderV2[] memory orders = new OrderV2[](1);
        orders[0] = volSellOrder;

        takeOrder(orders, SELL_ROUTE_NHS);
    }

    function takeOrder(OrderV2[] memory orders, bytes memory route) internal {
        vm.startPrank(NHS_APPROVED_EOA);

        // For all orders.
        uint256 inputIOIndex = 0;
        uint256 outputIOIndex = 0;

        TakeOrderConfigV2[] memory innerConfigs = new TakeOrderConfigV2[](orders.length);

        for (uint256 i = 0; i < innerConfigs.length; i++) {
            innerConfigs[i] = TakeOrderConfigV2(orders[i], inputIOIndex, outputIOIndex, new SignedContextV1[](0));
        }

        TakeOrdersConfigV2 memory takeOrdersConfig =
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, innerConfigs, route);

        POLYGON_NHS_ARB_CONTRACT.arb(takeOrdersConfig, 0);
        vm.stopPrank();
    }
}
