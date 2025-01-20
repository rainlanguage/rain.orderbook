// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {Clones} from "openzeppelin-contracts/contracts/proxy/Clones.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/deprecated/IExpressionDeployerV3.sol";
import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {Refundoor} from "test/util/concrete/Refundoor.sol";
import {
    FlashLendingMockOrderBook,
    OrderV3,
    TakeOrderConfigV3,
    IO,
    SignedContextV1,
    EvaluableV3
} from "test/util/concrete/FlashLendingMockOrderBook.sol";
import {OrderBookV4ArbConfigV2} from "src/concrete/arb/GenericPoolOrderBookV4ArbOrderTaker.sol";
import {TaskV1} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";

contract Token is ERC20 {
    constructor() ERC20("Token", "TKN") {}

    function mint(address receiver, uint256 amount) external {
        _mint(receiver, amount);
    }
}

abstract contract ArbTest is Test {
    IInterpreterV3 immutable iInterpreter;
    IInterpreterStoreV2 immutable iInterpreterStore;

    Token immutable iTakerInput;
    Token immutable iTakerOutput;
    address immutable iRefundoor;
    FlashLendingMockOrderBook immutable iOrderBook;
    address immutable iArb;

    /// Mimics the `Construct` event from `OrderBookV4ArbCommon`.
    event Construct(address sender, OrderBookV4ArbConfigV2 config);

    function expression() internal virtual returns (bytes memory) {
        return "";
    }

    function buildArb(OrderBookV4ArbConfigV2 memory config) internal virtual returns (address);

    constructor() {
        iInterpreter = IInterpreterV3(address(uint160(uint256(keccak256("interpreter.rain.test")))));
        vm.label(address(iInterpreter), "iInterpreter");
        iInterpreterStore = IInterpreterStoreV2(address(uint160(uint256(keccak256("interpreter.store.rain.test")))));
        vm.label(address(iInterpreterStore), "iInterpreterStore");

        iTakerInput = new Token();
        vm.label(address(iTakerInput), "iTakerInput");
        iTakerOutput = new Token();
        vm.label(address(iTakerOutput), "iTakerOutput");
        iRefundoor = address(new Refundoor());
        vm.label(iRefundoor, "iRefundoor");
        iOrderBook = new FlashLendingMockOrderBook();
        vm.label(address(iOrderBook), "iOrderBook");

        OrderBookV4ArbConfigV2 memory config = OrderBookV4ArbConfigV2(
            address(iOrderBook),
            TaskV1({
                evaluable: EvaluableV3(iInterpreter, iInterpreterStore, expression()),
                signedContext: new SignedContextV1[](0)
            }),
            abi.encode(iRefundoor)
        );

        vm.expectEmit();
        emit Construct(address(this), config);
        iArb = buildArb(config);
        vm.label(iArb, "iArb");
    }

    function buildTakeOrderConfig(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
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
