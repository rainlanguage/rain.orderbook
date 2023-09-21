// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookExternalRealTest, Vm} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {
    Order,
    SignedContextV1,
    TakeOrderConfig,
    TakeOrdersConfigV2,
    ZeroMaximumInput,
    IO,
    EvaluableConfigV2,
    OrderConfigV2
} from "src/interface/unstable/IOrderBookV3.sol";
import {IParserV1} from "rain.interpreter/src/interface/unstable/IParserV1.sol";

contract OrderBookTakeOrderMaximumInputTest is OrderBookExternalRealTest {
    /// If there is some live order(s) but the maxTakerInput is zero we error as
    /// the caller has full control over this, and it would cause none of the
    /// orders to be taken.
    function testTakeOrderNoopZeroMaxTakerInput(Order memory order, SignedContextV1 memory signedContext) external {
        vm.assume(order.validInputs.length > 0);
        vm.assume(order.validOutputs.length > 0);
        TakeOrderConfig[] memory orders = new TakeOrderConfig[](1);
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signedContext;
        orders[0] = TakeOrderConfig(order, 0, 0, signedContexts);
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, 0, type(uint256).max, orders, "");
        vm.expectRevert(ZeroMaximumInput.selector);
        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        (totalTakerInput, totalTakerOutput);
    }

    struct TestOrder {
        address owner;
        bytes orderString;
    }

    struct TestVault {
        address owner;
        address token;
        uint256 deposit;
        uint256 expect;
    }

    function checkTakeOrderMaximumInput(
        TestOrder[] memory testOrders,
        TestVault[] memory testVaults,
        uint256 maximumTakerInput,
        uint256 expectedTakerInput,
        uint256 expectedTakerOutput
    ) internal {
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));
        uint256 vaultId = 0;

        Order[] memory orders = new Order[](testOrders.length);

        for (uint256 i = 0; i < testOrders.length; i++) {
            {
                OrderConfigV2 memory orderConfig;
                {
                    (bytes memory bytecode, uint256[] memory constants) =
                        IParserV1(address(iDeployer)).parse(testOrders[i].orderString);
                    IO[] memory inputs = new IO[](1);
                    inputs[0] = IO(address(iToken0), 18, vaultId);
                    IO[] memory outputs = new IO[](1);
                    outputs[0] = IO(address(iToken1), 18, vaultId);
                    EvaluableConfigV2 memory evaluableConfig = EvaluableConfigV2(iDeployer, bytecode, constants);
                    orderConfig = OrderConfigV2(inputs, outputs, evaluableConfig, "");
                }

                vm.prank(testOrders[i].owner);
                vm.recordLogs();
                iOrderbook.addOrder(orderConfig);
                Vm.Log[] memory entries = vm.getRecordedLogs();
                assertEq(entries.length, 3);
                (,, Order memory order,) = abi.decode(entries[2].data, (address, address, Order, bytes32));
                orders[i] = order;
            }
        }

        for (uint256 i = 0; i < testVaults.length; i++) {
            if (testVaults[i].deposit > 0) {
                // Deposit the amount of tokens required to take the order.
                vm.mockCall(
                    address(iToken1),
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, testVaults[i].owner, address(iOrderbook), testVaults[i].deposit
                    ),
                    abi.encode(true)
                );
                vm.expectCall(
                    address(iToken1),
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector, testVaults[i].owner, address(iOrderbook), testVaults[i].deposit
                    ),
                    1
                );
                uint256 balanceBefore = iOrderbook.vaultBalance(testVaults[i].owner, testVaults[i].token, vaultId);
                vm.prank(testVaults[i].owner);
                iOrderbook.deposit(testVaults[i].token, vaultId, testVaults[i].deposit);
                assertEq(
                    iOrderbook.vaultBalance(testVaults[i].owner, testVaults[i].token, vaultId),
                    balanceBefore + testVaults[i].deposit,
                    "vaultBalance before"
                );
            }
        }

        vm.prank(bob);
        TakeOrderConfig[] memory takeOrders = new TakeOrderConfig[](orders.length);
        for (uint256 i = 0; i < orders.length; i++) {
            takeOrders[i] = TakeOrderConfig(orders[i], 0, 0, new SignedContextV1[](0));
        }
        TakeOrdersConfigV2 memory config = TakeOrdersConfigV2(0, maximumTakerInput, type(uint256).max, takeOrders, "");

        // Mock and expect the token transfers.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transfer.selector, bob, expectedTakerInput),
            abi.encode(true)
        );
        vm.expectCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transfer.selector, bob, expectedTakerInput),
            expectedTakerInput > 0 ? 1 : 0
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput),
            abi.encode(true)
        );
        vm.expectCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iOrderbook), expectedTakerOutput),
            expectedTakerOutput > 0 ? 1 : 0
        );

        (uint256 totalTakerInput, uint256 totalTakerOutput) = iOrderbook.takeOrders(config);
        assertEq(totalTakerInput, expectedTakerInput, "totalTakerInput");
        assertEq(totalTakerOutput, expectedTakerOutput, "totalTakerOutput");

        for (uint256 i = 0; i < testVaults.length; i++) {
            assertEq(
                iOrderbook.vaultBalance(testVaults[i].owner, testVaults[i].token, vaultId),
                testVaults[i].expect,
                "vaultBalance"
            );
        }
    }

    /// Add an order with unlimited maximum output and take it with a maximum
    /// input. Only the maximum input should be taken.
    function testTakeOrderMaximumInputSingleOrderUnlimitedMax(uint256 expectedTakerInput) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));

        expectedTakerInput = bound(expectedTakerInput, 1, type(uint128).max);
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:max-decimal18-value() 2e18;:;");

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), expectedTakerInput, 0);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, expectedTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// Add an order with less than the maximum output. Only the limit from the
    /// order should be taken.
    function testTakeOrderMaximumInputSingleOrderLessThanMaximumOutput(uint256 maximumTakerInput) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        maximumTakerInput = bound(maximumTakerInput, 1000, type(uint256).max);
        uint256 expectedTakerInput = 1000;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:1000 2e18;:;");

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), expectedTakerInput, 0);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// If the vault balance is less than both the maximum input and the order
    /// limit, the vault balance should be taken.
    function testTakeOrderMaximumInputSingleOrderLessThanMaximumInput(
        uint256 ownerDepositAmount,
        uint256 maximumTakerInput
    ) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit = 1000;
        ownerDepositAmount = bound(ownerDepositAmount, 0, orderLimit - 1);
        maximumTakerInput = bound(maximumTakerInput, 1000, type(uint256).max);
        uint256 expectedTakerInput = ownerDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:1000 2e18;:;");

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), ownerDepositAmount, 0);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// The deposit amount can be anything actually, the order taking should
    /// adjust accordingly, and leave any unspent deposited tokens in the vault.
    function testTakeOrderMaximumInputSingleAnyDeposit(uint256 ownerDepositAmount, uint256 maximumTakerInput)
        external
    {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit = 1000;

        TestOrder[] memory testOrders = new TestOrder[](1);
        testOrders[0] = TestOrder(owner, "_ _:1000 2e18;:;");

        maximumTakerInput = bound(maximumTakerInput, 1, type(uint256).max);
        // The expected input is the minimum of the maximum input and the order
        // limit.
        uint256 expectedTakerInput = maximumTakerInput < orderLimit ? maximumTakerInput : orderLimit;

        expectedTakerInput = expectedTakerInput < ownerDepositAmount ? expectedTakerInput : ownerDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), ownerDepositAmount, ownerDepositAmount - expectedTakerInput);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// The taker input can be sourced from multiple orders. Tests two orders
    /// that combined make up the maximum taker input. Both orders have the
    /// same owner.
    function testTakeOrderMaximumInputMultipleOrders(uint256 ownerDepositAmount, uint256 maximumTakerInput) external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        uint256 orderLimit = 1500;

        TestOrder[] memory testOrders = new TestOrder[](2);
        testOrders[0] = TestOrder(owner, "_ _:1000 2e18;:;");
        testOrders[1] = TestOrder(owner, "_ _:500 2e18;:;");

        maximumTakerInput = bound(maximumTakerInput, 1, type(uint256).max);
        // The expected input is the minimum of the maximum input and the order
        // limit.
        uint256 expectedTakerInput = maximumTakerInput < orderLimit ? maximumTakerInput : orderLimit;

        expectedTakerInput = expectedTakerInput < ownerDepositAmount ? expectedTakerInput : ownerDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        TestVault[] memory testVaults = new TestVault[](2);
        testVaults[0] = TestVault(owner, address(iToken1), ownerDepositAmount, ownerDepositAmount - expectedTakerInput);
        testVaults[1] = TestVault(owner, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }

    /// The taker input can be source from multiple orders with multiple owners.
    /// Tests two orders that combined make up the maximum taker input. Both
    /// orders have different owners.
    function testTakeOrderMaximumInputMultipleOrdersMultipleOwners(
        uint256 ownerOneDepositAmount,
        uint256 ownerTwoDepositAmount,
        uint256 maximumTakerInput
    ) external {
        // Avoid information free overflow.
        ownerTwoDepositAmount = bound(ownerTwoDepositAmount, 0, type(uint256).max - ownerOneDepositAmount);

        address ownerOne = address(uint160(uint256(keccak256("ownerOne.rain.test"))));
        address ownerTwo = address(uint160(uint256(keccak256("ownerTwo.rain.test"))));
        uint256 orderLimit = 1500;

        TestOrder[] memory testOrders = new TestOrder[](2);
        testOrders[0] = TestOrder(ownerOne, "_ _:1000 2e18;:;");
        testOrders[1] = TestOrder(ownerTwo, "_ _:500 2e18;:;");

        maximumTakerInput = bound(maximumTakerInput, 1, type(uint256).max);
        // The expected input is the minimum of the maximum input and the order
        // limit.
        uint256 expectedTakerInput = maximumTakerInput < orderLimit ? maximumTakerInput : orderLimit;

        uint256 totalDepositAmount = ownerOneDepositAmount + ownerTwoDepositAmount;
        expectedTakerInput = expectedTakerInput < totalDepositAmount ? expectedTakerInput : totalDepositAmount;
        uint256 expectedTakerOutput = expectedTakerInput * 2;

        // The first owner's deposit is fully used before the second owner's
        // deposit is used.
        TestVault[] memory testVaults = new TestVault[](2);

        uint256 ownerOneRemainingDeposit = expectedTakerInput > ownerOneDepositAmount ? 0 : ownerOneDepositAmount - expectedTakerInput;
        uint256 ownerOneExpectedTakerInput = ownerOneDepositAmount - ownerOneRemainingDeposit;
        testVaults[0] = TestVault(ownerOne, address(iToken1), ownerOneDepositAmount, ownerOneRemainingDeposit);

        uint256 ownerTwoExpectedTakerInput = expectedTakerInput > ownerOneExpectedTakerInput ? expectedTakerInput - ownerOneExpectedTakerInput : 0;
        uint256 ownerTwoRemainingDeposit = ownerTwoDepositAmount - ownerTwoExpectedTakerInput;
        testVaults[1] = TestVault(ownerTwo, address(iToken1), ownerTwoDepositAmount, ownerTwoRemainingDeposit);
        // testVaults[1] = TestVault(ownerOne, address(iToken0), 0, expectedTakerOutput);

        checkTakeOrderMaximumInput(testOrders, testVaults, maximumTakerInput, expectedTakerInput, expectedTakerOutput);
    }
}
