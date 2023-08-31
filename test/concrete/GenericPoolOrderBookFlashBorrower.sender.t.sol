// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/Clones.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibGenericPoolOrderBookV3FlashBorrowerConstants.sol";

import "src/concrete/GenericPoolOrderBookV3FlashBorrower.sol";
import "src/interface/unstable/IOrderBookV3.sol";

contract Token is ERC20 {
    constructor() ERC20("Token", "TKN") {}

    function mint(address receiver_, uint256 amount_) external {
        _mint(receiver_, amount_);
    }
}

contract MockOrderBook is IOrderBookV3 {
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        returns (bool)
    {
        receiver.onFlashLoan(msg.sender, token, amount, 0, data);
        return true;
    }

    function takeOrders(TakeOrdersConfigV2 calldata) external pure returns (uint256 totalInput, uint256 totalOutput) {
        return (0, 0);
    }

    function addOrder(OrderConfigV2 calldata) external pure returns (bool stateChanged) {
        return false;
    }

    function orderExists(bytes32) external pure returns (bool exists) {
        return false;
    }

    function clear(
        Order memory alice,
        Order memory bob,
        ClearConfig calldata clearConfig,
        SignedContextV1[] memory aliceSignedContextV1,
        SignedContextV1[] memory bobSignedContextV1
    ) external {}
    function deposit(address token, uint256 vaultId, uint256 amount) external {}
    function flashFee(address token, uint256 amount) external view returns (uint256) {}
    function maxFlashLoan(address token) external view returns (uint256) {}
    function removeOrder(Order calldata order) external returns (bool stateChanged) {}

    function vaultBalance(address owner, address token, uint256 id) external view returns (uint256 balance) {}
    function withdraw(address token, uint256 vaultId, uint256 targetAmount) external {}
}

contract Mock0xProxy {
    fallback() external {
        Address.sendValue(payable(msg.sender), address(this).balance);
    }
}

contract GenericPoolOrderBookFlashBorrowerTest is Test {
    address immutable deployer;
    address immutable implementation;

    constructor() {
        deployer = address(uint160(uint256(keccak256("deployer.rain.test"))));
        // All non-mocked calls will revert.
        vm.etch(deployer, REVERTING_MOCK_BYTECODE);
        vm.mockCall(
            deployer,
            abi.encodeWithSelector(IExpressionDeployerV2.deployExpression.selector),
            abi.encode(address(0), address(0), address(0))
        );
        bytes memory meta = vm.readFileBinary(GENERIC_POOL_ORDER_BOOK_FLASH_BORROWER_META_PATH);
        console2.log("GenericPoolOrderBookFlashBorrowerTest meta hash:");
        console2.logBytes32(keccak256(meta));
        implementation = address(
            new GenericPoolOrderBookV3FlashBorrower(DeployerDiscoverableMetaV2ConstructionConfig(
            deployer,
            meta
            ))
        );
    }

    function testTakeOrdersSender() public {
        MockOrderBook ob_ = new MockOrderBook();
        Mock0xProxy proxy_ = new Mock0xProxy();

        Token input_ = new Token();
        Token output_ = new Token();

        GenericPoolOrderBookV3FlashBorrower arb_ = GenericPoolOrderBookV3FlashBorrower(Clones.clone(implementation));
        arb_.initialize(
            abi.encode(
                OrderBookV3FlashBorrowerConfigV2(
                    address(ob_), EvaluableConfigV2(IExpressionDeployerV2(address(0)), "", new uint256[](0)), ""
                )
            )
        );

        arb_.arb(
            TakeOrdersConfigV2(
                address(output_), address(input_), 0, type(uint256).max, type(uint256).max, new TakeOrderConfig[](0), ""
            ),
            0,
            abi.encode(address(proxy_), address(proxy_), "")
        );
    }

    function testMinimumOutput(uint256 minimumOutput, uint256 mintAmount) public {
        vm.assume(minimumOutput > mintAmount);
        MockOrderBook ob = new MockOrderBook();
        Mock0xProxy proxy = new Mock0xProxy();

        Token input = new Token();
        Token output = new Token();

        GenericPoolOrderBookV3FlashBorrower arb = GenericPoolOrderBookV3FlashBorrower(Clones.clone(implementation));
        arb.initialize(
            abi.encode(
                OrderBookV3FlashBorrowerConfigV2(
                    address(ob), EvaluableConfigV2(IExpressionDeployerV2(address(0)), "", new uint256[](0)), ""
                )
            )
        );

        output.mint(address(arb), mintAmount);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        arb.arb(
            TakeOrdersConfigV2(
                address(output), address(input), 0, type(uint256).max, type(uint256).max, new TakeOrderConfig[](0), ""
            ),
            minimumOutput,
            abi.encode(address(proxy), address(proxy), "")
        );
    }

    // Allow receiving funds at end of arb.
    fallback() external {}
}
