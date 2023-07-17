// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/Clones.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

import "test/util/lib/LibTestConstants.sol";
import "test/util/lib/LibGenericPoolOrderBookFlashBorrowerConstants.sol";

import "src/concrete/GenericPoolOrderBookFlashBorrower.sol";
import "src/interface/IOrderBookV2.sol";

contract Token is ERC20 {
    constructor() ERC20("Token", "TKN") {}

    function mint(address receiver_, uint256 amount_) external {
        _mint(receiver_, amount_);
    }
}

contract MockOrderBook is IOrderBookV2 {
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        returns (bool)
    {
        receiver.onFlashLoan(msg.sender, token, amount, 0, data);
        return true;
    }

    function takeOrders(TakeOrdersConfig calldata) external pure returns (uint256 totalInput, uint256 totalOutput) {
        return (0, 0);
    }

    function addOrder(OrderConfig calldata config) external {}
    function clear(
        Order memory alice,
        Order memory bob,
        ClearConfig calldata clearConfig,
        SignedContextV1[] memory aliceSignedContextV1,
        SignedContextV1[] memory bobSignedContextV1
    ) external {}
    function deposit(DepositConfig calldata config) external {}
    function flashFee(address token, uint256 amount) external view returns (uint256) {}
    function maxFlashLoan(address token) external view returns (uint256) {}
    function removeOrder(Order calldata order) external {}

    function vaultBalance(address owner, address token, uint256 id) external view returns (uint256 balance) {}
    function withdraw(WithdrawConfig calldata config) external {}
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
            abi.encodeWithSelector(IExpressionDeployerV1.deployExpression.selector),
            abi.encode(address(0), address(0), address(0))
        );
        bytes memory meta = vm.readFileBinary(GENERIC_POOL_ORDER_BOOK_FLASH_BORROWER_META_PATH);
        console2.logBytes32(keccak256(meta));
        implementation = address(
            new GenericPoolOrderBookFlashBorrower(DeployerDiscoverableMetaV1ConstructionConfig(
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

        GenericPoolOrderBookFlashBorrower arb_ = GenericPoolOrderBookFlashBorrower(Clones.clone(implementation));
        arb_.initialize(
            abi.encode(
                OrderBookFlashBorrowerConfig(
                    address(ob_),
                    EvaluableConfig(IExpressionDeployerV1(address(0)), new bytes[](0), new uint256[](0)),
                    ""
                )
            )
        );

        arb_.arb(
            TakeOrdersConfig(
                address(output_), address(input_), 0, type(uint256).max, type(uint256).max, new TakeOrderConfig[](0)
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

        GenericPoolOrderBookFlashBorrower arb = GenericPoolOrderBookFlashBorrower(Clones.clone(implementation));
        arb.initialize(
            abi.encode(
                OrderBookFlashBorrowerConfig(
                    address(ob),
                    EvaluableConfig(IExpressionDeployerV1(address(0)), new bytes[](0), new uint256[](0)),
                    ""
                )
            )
        );

        output.mint(address(arb), mintAmount);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        arb.arb(
            TakeOrdersConfig(
                address(output), address(input), 0, type(uint256).max, type(uint256).max, new TakeOrderConfig[](0)
            ),
            minimumOutput,
            abi.encode(address(proxy), address(proxy), "")
        );
    }

    // Allow receiving funds at end of arb.
    fallback() external {}
}
