// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {console2} from "forge-std/console2.sol";
import {
    IOrderBookV6,
    OrderV4,
    OrderConfigV4,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    TaskV2,
    Float,
    IOV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV6.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {OrderBookV6} from "src/concrete/ob/OrderBookV6.sol";
import {RaindexRouterOrderBookV6Arb, RouteLeg, RouteLegType} from "src/concrete/arb/RaindexRouterOrderBookV6Arb.sol";
import {OrderBookV6ArbConfig} from "src/abstract/OrderBookV6ArbCommon.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {
    EvaluableV4,
    IInterpreterStoreV3,
    IInterpreterV4,
    SignedContextV1
} from "rain.interpreter.interface/interface/unstable/IInterpreterCallerV4.sol";
import {TOFUTokenDecimals, LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/concrete/TOFUTokenDecimals.sol";
import {RainterpreterParser} from "rain.interpreter/concrete/RainterpreterParser.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {Rainterpreter} from "rain.interpreter/concrete/Rainterpreter.sol";
import {RainterpreterStore} from "rain.interpreter/concrete/RainterpreterStore.sol";
import {
    RainterpreterExpressionDeployer,
    RainterpreterExpressionDeployerConstructionConfigV2
} from "rain.interpreter/concrete/RainterpreterExpressionDeployer.sol";
import {IRouteProcessor} from "sushixswap-v2/src/interfaces/IRouteProcessor.sol";

/// @title RaindexRouterOrderBookV6ArbTest
/// @notice Tests for the arb4() method in OrderBookV6RaindexRouter
contract RaindexRouterOrderBookV6ArbTest is Test {
    using LibDecimalFloat for Float;

    OrderBookV6 internal orderBook;
    RaindexRouterOrderBookV6Arb internal router;

    address internal alice;
    address internal bob;
    address internal arber;

    address internal tokenA;
    address internal tokenB;
    address internal tokenC;

    uint256 internal constant INITIAL_BALANCE = 1000e18;
    bytes32 internal constant VAULT_ID = keccak256("vault");

    IInterpreterV4 internal immutable iInterpreter;
    IInterpreterStoreV3 internal immutable iStore;
    RainterpreterParser internal immutable iParser;
    IParserV2 internal immutable iParserV2;
    IOrderBookV6 internal immutable iOrderbook;
    MockSushiRP internal immutable sushi;

    constructor() {
        // Put the TOFU decimals contract in place so that any calls to it
        // succeed. This is because we don't have zoltu here.
        vm.etch(address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT), type(TOFUTokenDecimals).runtimeCode);

        iInterpreter = IInterpreterV4(new Rainterpreter());
        iStore = IInterpreterStoreV3(new RainterpreterStore());
        iParser = new RainterpreterParser();
        iParserV2 = new RainterpreterExpressionDeployer(
            RainterpreterExpressionDeployerConstructionConfigV2({
                interpreter: address(iInterpreter),
                store: address(iStore),
                parser: address(iParser)
            })
        );

        iOrderbook = IOrderBookV6(address(new OrderBookV6()));
        sushi = new MockSushiRP();
    }

    function setUp() public {
        // Deploy core contracts
        orderBook = new OrderBookV6();

        // Create test accounts
        alice = makeAddr("alice");
        bob = makeAddr("bob");
        arber = makeAddr("arber");

        // Deploy mock ERC20 tokens
        tokenA = address(new Token("Token A", "TKNA"));
        tokenB = address(new Token("Token B", "TKNB"));
        tokenC = address(new Token("Token C", "TKNC"));

        // Setup router
        OrderBookV6ArbConfig memory config = OrderBookV6ArbConfig({
            orderBook: address(orderBook),
            task: TaskV2({
                evaluable: EvaluableV4({interpreter: iInterpreter, store: iStore, bytecode: new bytes(0)}),
                signedContext: new SignedContextV1[](0)
            }),
            implementationData: new bytes(0)
        });

        router = new RaindexRouterOrderBookV6Arb(config);

        // Fund accounts
        deal(tokenA, alice, INITIAL_BALANCE);
        deal(tokenB, bob, INITIAL_BALANCE);

        // Fund sushi RP for the swap
        deal(tokenC, address(sushi), INITIAL_BALANCE);

        // Fund orderbook with some tokens
        deal(tokenA, address(orderBook), INITIAL_BALANCE);
        deal(tokenB, address(orderBook), INITIAL_BALANCE);
        deal(tokenC, address(orderBook), INITIAL_BALANCE);
    }

    /// @dev Test successful arb4() with simple two-order arbitrage
    function testArb4Success() public {
        // Alice creates order: sell tokenA for tokenB at 1:1.01 ratio
        OrderV4 memory aliceOrder = createOrder(
            alice,
            tokenB, // input
            tokenA, // output
            "_ _: 100 0.2;:;"
        );

        // Bob creates order: sell tokenB for tokenA at 1:0.99 ratio
        OrderV4 memory bobOrder = createOrder(
            bob,
            tokenC, // input
            tokenB, // output
            "_ _: 100 0.5;:;"
        );

        // Deposit tokens into vaults
        vm.startPrank(alice);
        IERC20(tokenA).approve(address(orderBook), type(uint256).max);
        orderBook.deposit4(
            tokenA, VAULT_ID, LibDecimalFloat.fromFixedDecimalLosslessPacked(100000000000000000000, 18), new TaskV2[](0)
        );
        vm.stopPrank();

        vm.startPrank(bob);
        IERC20(tokenB).approve(address(orderBook), type(uint256).max);
        orderBook.deposit4(
            tokenB, VAULT_ID, LibDecimalFloat.fromFixedDecimalLosslessPacked(100000000000000000000, 18), new TaskV2[](0)
        );
        vm.stopPrank();

        // Add orders
        vm.prank(alice);
        orderBook.addOrder4(createOrderConfig(aliceOrder), new TaskV2[](0));

        vm.prank(bob);
        orderBook.addOrder4(createOrderConfig(bobOrder), new TaskV2[](0));

        // Create take orders configs for arbitrage
        TakeOrdersConfigV5 memory startTakeOrders = createTakeOrdersConfig(
            aliceOrder,
            0, // inputIOIndex
            0, // outputIOIndex
            LibDecimalFloat.FLOAT_MAX_POSITIVE_VALUE, // maximumIO
            LibDecimalFloat.FLOAT_MAX_POSITIVE_VALUE, // maximumIORatio
            false // IOIsInput
        );

        TakeOrdersConfigV5 memory endTakeOrders = createTakeOrdersConfig(
            bobOrder,
            0, // inputIOIndex
            0, // outputIOIndex
            LibDecimalFloat.FLOAT_MAX_POSITIVE_VALUE, // maximumIO
            LibDecimalFloat.FLOAT_MAX_POSITIVE_VALUE, // maximumIORatio
            false // IOIsInput
        );
        TakeOrdersConfigV5[] memory takeOrders = new TakeOrdersConfigV5[](2);
        takeOrders[0] = startTakeOrders;
        takeOrders[1] = endTakeOrders;

        TaskV2 memory task = TaskV2({
            evaluable: EvaluableV4({interpreter: iInterpreter, store: iStore, bytecode: new bytes(0)}),
            signedContext: new SignedContextV1[](0)
        });

        RouteLeg[] memory routeLegs = new RouteLeg[](1);
        routeLegs[0] = RouteLeg({
            routeLegType: RouteLegType.SUSHI,
            destination: address(sushi),
            data: abi.encode(tokenA, tokenC, new bytes(0))
        });
        bytes memory exchangeData = abi.encode(routeLegs);

        // Record balances before
        uint256 orderBookTokenABefore = IERC20(tokenA).balanceOf(address(orderBook));
        uint256 orderBookTokenBBefore = IERC20(tokenB).balanceOf(address(orderBook));
        uint256 orderBookTokenCBefore = IERC20(tokenC).balanceOf(address(orderBook));
        uint256 arberTokenABefore = IERC20(tokenA).balanceOf(arber);
        uint256 arberTokenBBefore = IERC20(tokenB).balanceOf(arber);
        uint256 arberTokenCBefore = IERC20(tokenC).balanceOf(arber);

        // Execute arbitrage
        vm.prank(arber);
        router.arb4(orderBook, takeOrders, exchangeData, task);

        // Verify balances changed
        uint256 orderBookTokenAAfter = IERC20(tokenA).balanceOf(address(orderBook));
        uint256 orderBookTokenBAfter = IERC20(tokenB).balanceOf(address(orderBook));
        uint256 orderBookTokenCAfter = IERC20(tokenC).balanceOf(address(orderBook));
        uint256 arberTokenAAfter = IERC20(tokenA).balanceOf(arber);
        uint256 arberTokenBAfter = IERC20(tokenB).balanceOf(arber);
        uint256 arberTokenCAfter = IERC20(tokenC).balanceOf(arber);

        assertNotEq(orderBookTokenABefore, orderBookTokenAAfter, "TokenA balance should change");
        assertNotEq(orderBookTokenBBefore, orderBookTokenBAfter, "TokenB balance should change");
        assertNotEq(orderBookTokenCBefore, orderBookTokenCAfter, "TokenC balance should change");

        assertEq(arberTokenABefore, arberTokenAAfter, "arber TokenA balance should NOT change");

        assertNotEq(arberTokenBBefore, arberTokenBAfter, "arber TokenB balance should change");
        assertNotEq(arberTokenCBefore, arberTokenCAfter, "arber TokenC balance should change");

        assertEq(arberTokenBAfter, 8e19, "expected 80 token B for arber bounty");
        assertEq(arberTokenCAfter, 5e19, "expected 50 token C for arber bounty");

        Float aliceOutputVaultBalanceFloat = orderBook.vaultBalance2(alice, tokenA, VAULT_ID);
        Float aliceInputVaultBalanceFloat = orderBook.vaultBalance2(alice, tokenB, VAULT_ID);
        uint256 aliceOutputVaultBalance = LibDecimalFloat.toFixedDecimalLossless(aliceOutputVaultBalanceFloat, 18);
        uint256 aliceInputVaultBalance = LibDecimalFloat.toFixedDecimalLossless(aliceInputVaultBalanceFloat, 18);
        assertEq(aliceOutputVaultBalance, 0, "expected zero vault balance for alice output vault");
        assertEq(aliceInputVaultBalance, 2e19, "expected 20 token B vault balance for alice output vault");

        Float bobOutputVaultBalanceFloat = orderBook.vaultBalance2(bob, tokenB, VAULT_ID);
        Float bobInputVaultBalanceFloat = orderBook.vaultBalance2(bob, tokenC, VAULT_ID);
        uint256 bobOutputVaultBalance = LibDecimalFloat.toFixedDecimalLossless(bobOutputVaultBalanceFloat, 18);
        uint256 bobInputVaultBalance = LibDecimalFloat.toFixedDecimalLossless(bobInputVaultBalanceFloat, 18);
        assertEq(bobOutputVaultBalance, 0, "expected zero vault balance for bob output vault");
        assertEq(bobInputVaultBalance, 5e19, "expected 50 token C vault balance for alice output vault");
    }

    // Helper functions
    function createOrder(address owner, address inputToken, address outputToken, string memory rl)
        internal
        view
        returns (OrderV4 memory)
    {
        IOV2[] memory validInputs = new IOV2[](1);
        validInputs[0] = IOV2({token: inputToken, vaultId: VAULT_ID});

        IOV2[] memory validOutputs = new IOV2[](1);
        validOutputs[0] = IOV2({token: outputToken, vaultId: VAULT_ID});

        return OrderV4({
            owner: owner,
            evaluable: EvaluableV4({interpreter: iInterpreter, store: iStore, bytecode: iParserV2.parse2(bytes(rl))}),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: bytes32(0)
        });
    }

    function createOrderConfig(OrderV4 memory order) internal pure returns (OrderConfigV4 memory) {
        return OrderConfigV4({
            validInputs: order.validInputs,
            validOutputs: order.validOutputs,
            evaluable: order.evaluable,
            meta: new bytes(0),
            nonce: order.nonce,
            secret: bytes32(0)
        });
    }

    function createTakeOrdersConfig(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        Float maximumIO,
        Float maximumIORatio,
        bool ioIsInput
    ) internal pure returns (TakeOrdersConfigV5 memory) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4({
            order: order,
            inputIOIndex: inputIOIndex,
            outputIOIndex: outputIOIndex,
            signedContext: new SignedContextV1[](0)
        });

        return TakeOrdersConfigV5({
            minimumIO: Float.wrap(0),
            maximumIO: maximumIO,
            maximumIORatio: maximumIORatio,
            IOIsInput: ioIsInput,
            orders: orders,
            data: new bytes(0)
        });
    }
}

contract Token is ERC20 {
    constructor(string memory name_, string memory symbol_) ERC20(name_, symbol_) {}

    function mint(address receiver, uint256 amount) external {
        _mint(receiver, amount);
    }
}

contract MockSushiRP is IRouteProcessor {
    function processRoute(
        address tokenIn,
        uint256 amountIn,
        address tokenOut,
        uint256 amountOutMin,
        address to,
        bytes calldata route
    ) external payable returns (uint256 amountOut) {
        IERC20(tokenIn).transferFrom(msg.sender, address(this), amountIn);
        IERC20(tokenOut).approve(address(this), 0);
        IERC20(tokenOut).approve(address(this), type(uint256).max);
        IERC20(tokenOut).transferFrom(address(this), msg.sender, amountIn);
        IERC20(tokenOut).approve(address(this), 0);
        return amountIn;
    }
}
