// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Test, console2} from "forge-std/Test.sol";
import {Clones} from "openzeppelin-contracts/contracts/proxy/Clones.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/IExpressionDeployerV3.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {Refundoor} from "test/util/concrete/Refundoor.sol";
import {
    FlashLendingMockOrderBook,
    OrderV3,
    TakeOrderConfigV3,
    IO,
    SignedContextV1
} from "test/util/concrete/FlashLendingMockOrderBook.sol";

contract Token is ERC20 {
    constructor() ERC20("Token", "TKN") {}

    function mint(address receiver, uint256 amount) external {
        _mint(receiver, amount);
    }
}

struct ArbTestConstructorConfig {
    address deployer;
    address implementation;
}

abstract contract ArbTest is Test {
    address immutable iDeployer;
    address immutable iImplementation;
    Token immutable iTakerInput;
    Token immutable iTakerOutput;
    address immutable iRefundoor;
    FlashLendingMockOrderBook immutable iOrderBook;
    address iArb;

    constructor(ArbTestConstructorConfig memory config) {
        iDeployer = config.deployer;
        vm.label(iDeployer, "iDeployer");
        iImplementation = config.implementation;
        vm.label(iImplementation, "iImplementation");
        iArb = Clones.clone(iImplementation);
        vm.label(iArb, "iArb");
        iTakerInput = new Token();
        vm.label(address(iTakerInput), "iTakerInput");
        iTakerOutput = new Token();
        vm.label(address(iTakerOutput), "iTakerOutput");
        iRefundoor = address(new Refundoor());
        vm.label(iRefundoor, "iRefundoor");
        iOrderBook = new FlashLendingMockOrderBook();
        vm.label(address(iOrderBook), "iOrderBook");
    }

    function buildConstructorConfig() internal returns (address deployer) {
        deployer = address(uint160(uint256(keccak256("deployer.rain.test"))));
        vm.label(deployer, "deployer");
        // All non-mocked calls will revert.
        vm.etch(deployer, REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            deployer,
            abi.encodeWithSelector(IExpressionDeployerV3.deployExpression2.selector),
            // Don't need any io for the "before arb" expression.
            abi.encode(address(0), address(0), address(0), "0000")
        );
    }

    function buildTakeOrderConfig(OrderV2 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        internal
        view
        returns (TakeOrderConfigV3[] memory)
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

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](1);
        orders[0] = TakeOrderConfigV3(order, inputIOIndex, outputIOIndex, new SignedContextV1[](0));
        return orders;
    }

    // Allow receiving funds at end of arb.
    fallback() external {}
}
