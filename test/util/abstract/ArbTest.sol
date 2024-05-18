// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test} from "forge-std/Test.sol";
import {Clones} from "openzeppelin-contracts/contracts/proxy/Clones.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/IExpressionDeployerV3.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {Refundoor} from "test/util/concrete/Refundoor.sol";
import {
    FlashLendingMockOrderBook,
    OrderV2,
    TakeOrderConfigV2,
    IO,
    SignedContextV1
} from "test/util/concrete/FlashLendingMockOrderBook.sol";
import {OrderBookV3ArbConfigV1} from "src/concrete/arb/GenericPoolOrderBookV3ArbOrderTaker.sol";
import {EvaluableConfigV3} from "rain.orderbook.interface/interface/IOrderBookV3.sol";

contract Token is ERC20 {
    constructor() ERC20("Token", "TKN") {}

    function mint(address receiver, uint256 amount) external {
        _mint(receiver, amount);
    }
}

abstract contract ArbTest is Test {
    address immutable iDeployer;
    Token immutable iTakerInput;
    Token immutable iTakerOutput;
    address immutable iRefundoor;
    FlashLendingMockOrderBook immutable iOrderBook;
    address immutable iArb;

    function buildArb(OrderBookV3ArbConfigV1 memory config) internal virtual returns (address);

    constructor() {
        iDeployer = address(uint160(uint256(keccak256("deployer.rain.test"))));
        vm.label(iDeployer, "iDeployer");
        iTakerInput = new Token();
        vm.label(address(iTakerInput), "iTakerInput");
        iTakerOutput = new Token();
        vm.label(address(iTakerOutput), "iTakerOutput");
        iRefundoor = address(new Refundoor());
        vm.label(iRefundoor, "iRefundoor");
        iOrderBook = new FlashLendingMockOrderBook();
        vm.label(address(iOrderBook), "iOrderBook");

        iArb = buildArb(
            OrderBookV3ArbConfigV1(
                address(iOrderBook),
                EvaluableConfigV3(IExpressionDeployerV3(address(0)), "", new uint256[](0)),
                abi.encode(iRefundoor)
            )
        );
        vm.label(iArb, "iArb");
    }

    function buildTakeOrderConfig(OrderV2 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        internal
        view
        returns (TakeOrderConfigV2[] memory)
    {
        if (order.validInputs.length == 0) {
            order.validInputs = new IO[](1);
        }
        if (order.validOutputs.length == 0) {
            order.validOutputs = new IO[](1);
        }
        inputIOIndex = bound(inputIOIndex, 0, order.validInputs.length - 1);
        outputIOIndex = bound(outputIOIndex, 0, order.validOutputs.length - 1);

        order.validInputs[inputIOIndex].token = address(iTakerOutput);
        order.validOutputs[outputIOIndex].token = address(iTakerInput);

        TakeOrderConfigV2[] memory orders = new TakeOrderConfigV2[](1);
        orders[0] = TakeOrderConfigV2(order, inputIOIndex, outputIOIndex, new SignedContextV1[](0));
        return orders;
    }

    // Allow receiving funds at end of arb.
    fallback() external {}
}
