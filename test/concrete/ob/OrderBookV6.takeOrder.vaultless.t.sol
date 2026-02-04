// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {console2} from "forge-std/console2.sol";
import {
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

/// @title OrderBookV6TakeOrderVaultlessTest
/// @notice Tests for the taking vaultles order
contract OrderBookV6TakeOrderVaultlessTest is Test {
    using LibDecimalFloat for Float;

    address internal alice;
    address internal taker;

    address internal tokenA;
    address internal tokenB;

    uint256 internal constant INITIAL_BALANCE = 1000e18;
    bytes32 internal constant VAULT_ID = 0; // vaultless

    IInterpreterV4 internal immutable iInterpreter;
    IInterpreterStoreV3 internal immutable iStore;
    RainterpreterParser internal immutable iParser;
    IParserV2 internal immutable iParserV2;
    OrderBookV6 internal immutable orderBook;

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

        orderBook = new OrderBookV6();
    }

    function setUp() public {
        // Create test accounts
        alice = makeAddr("alice");
        taker = makeAddr("taker");

        // Deploy mock ERC20 tokens
        tokenA = address(new Token("Token A", "TKNA"));
        tokenB = address(new Token2("Token B", "TKNB"));

        // Fund alice and taker wallets
        deal(tokenA, alice, INITIAL_BALANCE);
        deal(tokenB, taker, INITIAL_BALANCE);

        // uncomment to set some initial balance for
        // orderbook which will cause the test to pass
        // deal(tokenB, address(orderBook), INITIAL_BALANCE);
    }

    /// @dev Test taking vaultless order
    function testVaultLessTakeOrder() public {
        // alice creates order: sell 100 tokenA for tokenB at .02 ratio
        OrderV4 memory aliceOrder = createOrder(
            alice,
            tokenB, // input
            tokenA, // output
            "_ _: 100 0.2;:;"
        );

        // Approve token A spend for alice
        vm.startPrank(alice);
        IERC20(tokenA).approve(address(orderBook), type(uint256).max);
        vm.stopPrank();

        // Add orders
        vm.prank(alice);
        orderBook.addOrder4(createOrderConfig(aliceOrder), new TaskV2[](0));

        // Approve token B spend for taker
        vm.startPrank(taker);
        IERC20(tokenB).approve(address(orderBook), type(uint256).max);
        vm.stopPrank();

        // Create take orders configs for arbitrage
        TakeOrdersConfigV5 memory takeOrders = createTakeOrdersConfig(
            aliceOrder,
            0, // inputIOIndex
            0, // outputIOIndex
            LibDecimalFloat.FLOAT_MAX_POSITIVE_VALUE, // maximumIO
            LibDecimalFloat.FLOAT_MAX_POSITIVE_VALUE, // maximumIORatio
            true // IOIsInput
        );

        // Record balances before
        uint256 aliceTokenABefore = IERC20(tokenA).balanceOf(alice);
        uint256 aliceTokenBBefore = IERC20(tokenB).balanceOf(alice);
        uint256 takerTokenABefore = IERC20(tokenA).balanceOf(taker);
        uint256 takerTokenBBefore = IERC20(tokenB).balanceOf(taker);

        // Execute takeOrders4() by taker
        vm.prank(taker);
        orderBook.takeOrders4(takeOrders);

        // Verify balances changed
        uint256 aliceTokenAAfter = IERC20(tokenA).balanceOf(alice);
        uint256 aliceTokenBAfter = IERC20(tokenB).balanceOf(alice);
        uint256 takerTokenAAfter = IERC20(tokenA).balanceOf(taker);
        uint256 takerTokenBAfter = IERC20(tokenB).balanceOf(taker);

        // alice token A and B balances should change
        assertNotEq(aliceTokenABefore, aliceTokenAAfter, "alice TokenA balance should change");
        assertNotEq(aliceTokenBBefore, aliceTokenBAfter, "alice TokenB balance should change");
        assertEq(aliceTokenAAfter, INITIAL_BALANCE - 100e18, "unexpected alice TokenA balance"); // alice sold 100 token A
        assertEq(aliceTokenBAfter, 20e18, "unexpected alice TokenB balance"); // alice bought 20 token B

        // taker token A and B balances should change
        assertNotEq(takerTokenABefore, takerTokenAAfter, "taker TokenA balance should change");
        assertNotEq(takerTokenBBefore, takerTokenBAfter, "taker TokenB balance should change");
        assertEq(takerTokenAAfter, 100e18, "unexpected taker TokenA balance"); // taker bought 100 token A
        assertEq(takerTokenBAfter, INITIAL_BALANCE - 20e18, "unexpected taker TokenA balance"); // taker sold 20 token B
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

contract Token2 is ERC20 {
    constructor(string memory name_, string memory symbol_) ERC20(name_, symbol_) {}

    function mint(address receiver, uint256 amount) external {
        _mint(receiver, amount);
    }

    function a() external {}
}
