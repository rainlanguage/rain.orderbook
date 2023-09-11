// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/Clones.sol";

import "test/util/lib/LibTestConstants.sol";
import {DeployerDiscoverableMetaV2ConstructionConfig} from
    "rain.interpreter/src/abstract/DeployerDiscoverableMetaV2.sol";
import {IExpressionDeployerV2} from "rain.interpreter/src/interface/unstable/IExpressionDeployerV2.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "test/util/concrete/Refundoor.sol";
import "test/util/concrete/FlashLendingMockOrderBook.sol";

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
        iImplementation = config.implementation;
        iArb = Clones.clone(iImplementation);
        iTakerInput = new Token();
        iTakerOutput = new Token();
        iRefundoor = address(new Refundoor());
        iOrderBook = new FlashLendingMockOrderBook();
    }

    function buildConstructorConfig(string memory metaPath)
        internal
        returns (address deployer, DeployerDiscoverableMetaV2ConstructionConfig memory config)
    {
        deployer = address(uint160(uint256(keccak256("deployer.rain.test"))));
        // All non-mocked calls will revert.
        vm.etch(deployer, REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            deployer,
            abi.encodeWithSelector(IExpressionDeployerV2.deployExpression.selector),
            abi.encode(address(0), address(0), address(0))
        );
        bytes memory meta = vm.readFileBinary(metaPath);
        console2.log("RouteProcessorOrderBookV3ArbOrderTakerTest meta hash:");
        console2.logBytes32(keccak256(meta));
        config = DeployerDiscoverableMetaV2ConstructionConfig(deployer, meta);
    }

    function buildTakeOrderConfig(Order memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        internal
        view
        returns (TakeOrderConfig[] memory)
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

        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        orders[0] = TakeOrderConfig(order, inputIOIndex, outputIOIndex, new SignedContextV1[](0));
        return orders;
    }

    // Allow receiving funds at end of arb.
    fallback() external {}
}
