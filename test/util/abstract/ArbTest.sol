// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Test.sol";

import "test/util/lib/LibTestConstants.sol";
import {DeployerDiscoverableMetaV2ConstructionConfig} from
    "rain.interpreter/src/abstract/DeployerDiscoverableMetaV2.sol";
import {IExpressionDeployerV2} from "rain.interpreter/src/interface/unstable/IExpressionDeployerV2.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "test/util/concrete/Refundoor.sol";

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

    constructor(ArbTestConstructorConfig memory config) {
        iDeployer = config.deployer;
        iImplementation = config.implementation;
        iTakerInput = new Token();
        iTakerOutput = new Token();
        iRefundoor = address(new Refundoor());
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

    // Allow receiving funds at end of arb.
    fallback() external {}
}
