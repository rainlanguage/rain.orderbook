// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {ERC165, IERC165} from "lib/openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {ReentrancyGuard} from "lib/openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {Initializable} from "lib/openzeppelin-contracts/contracts/proxy/utils/Initializable.sol";
import {SafeERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "lib/openzeppelin-contracts/contracts/utils/Address.sol";
import {
    DeployerDiscoverableMetaV2,
    DeployerDiscoverableMetaV2ConstructionConfig,
    LibMeta
} from "lib/rain.interpreter/src/abstract/DeployerDiscoverableMetaV2.sol";
import "lib/rain.factory/src/interface/ICloneableV2.sol";
import "lib/rain.interpreter/src/lib/caller/LibContext.sol";
import "lib/rain.interpreter/src/lib/caller/LibEncodedDispatch.sol";
import "lib/rain.interpreter/src/lib/bytecode/LibBytecode.sol";

import "../interface/unstable/IOrderBookV3.sol";
import "../interface/unstable/IOrderBookV3OrderTaker.sol";
import "../interface/unstable/IOrderBookV3ArbOrderTaker.sol";

import "./OrderBookV3ArbCommon.sol";

/// Config for `OrderBookV3ArbOrderTakerConfigV1` to initialize.
/// @param orderBook The `IOrderBookV3` to use for `takeOrders`.
/// @param evaluableConfig The config to eval for access control to arb.
/// @param implementationData Arbitrary bytes to pass to the implementation in
/// the `beforeInitialize` hook.
struct OrderBookV3ArbOrderTakerConfigV1 {
    address orderBook;
    EvaluableConfigV2 evaluableConfig;
    bytes implementationData;
}

/// @dev "Before arb" is evaluabled before the arb is executed. Ostensibly this
/// is to allow for access control to the arb, the return values are ignored.
SourceIndex constant BEFORE_ARB_SOURCE_INDEX = SourceIndex.wrap(0);
/// @dev "Before arb" has no return values.
uint256 constant BEFORE_ARB_MIN_OUTPUTS = 0;
/// @dev "Before arb" has no return values.
uint16 constant BEFORE_ARB_MAX_OUTPUTS = 0;

abstract contract OrderBookV3ArbOrderTaker is
    IOrderBookV3ArbOrderTaker,
    ReentrancyGuard,
    Initializable,
    ICloneableV2,
    DeployerDiscoverableMetaV2,
    ERC165
{
    using SafeERC20 for IERC20;

    event Initialize(address sender, OrderBookV3ArbOrderTakerConfigV1 config);

    IOrderBookV3 public sOrderBook;
    EncodedDispatch public sI9rDispatch;
    IInterpreterV1 public sI9r;
    IInterpreterStoreV1 public sI9rStore;

    constructor(bytes32 metaHash, DeployerDiscoverableMetaV2ConstructionConfig memory config)
        DeployerDiscoverableMetaV2(metaHash, config)
    {
        _disableInitializers();
    }

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return interfaceId == type(IOrderBookV3OrderTaker).interfaceId || interfaceId == type(ICloneableV2).interfaceId
            || super.supportsInterface(interfaceId);
    }

    function _beforeInitialize(bytes memory data) internal virtual {}

    /// Ensure the contract is not initializing.
    modifier onlyNotInitializing() {
        if (_isInitializing()) {
            revert Initializing();
        }
        _;
    }

    function initialize(OrderBookV3ArbOrderTakerConfigV1 calldata) external pure returns (bytes32) {
        revert InitializeSignatureFn();
    }

    /// @inheritdoc ICloneableV2
    function initialize(bytes memory data) external initializer nonReentrant returns (bytes32) {
        OrderBookV3ArbOrderTakerConfigV1 memory config = abi.decode(data, (OrderBookV3ArbOrderTakerConfigV1));

        // Dispatch the hook before any external calls are made.
        _beforeInitialize(config.implementationData);

        // @todo this could be paramaterised on `arb`.
        sOrderBook = IOrderBookV3(config.orderBook);

        // Emit events before any external calls are made.
        emit Initialize(msg.sender, config);

        // If there are any sources to eval then initialize the dispatch,
        // otherwise it will remain 0 and we can skip evaluation on `arb`.
        if (LibBytecode.sourceCount(config.evaluableConfig.bytecode) > 0) {
            address expression;

            uint256[] memory entrypoints = new uint256[](1);
            entrypoints[SourceIndex.unwrap(BEFORE_ARB_SOURCE_INDEX)] = BEFORE_ARB_MIN_OUTPUTS;

            // We have to trust the deployer because it produces the expression
            // address for dispatch anyway.
            // All external functions on this contract have `onlyNotInitializing`
            // modifier on them so can't be reentered here anyway.
            //slither-disable-next-line reentrancy-benign
            (sI9r, sI9rStore, expression) = config.evaluableConfig.deployer.deployExpression(
                config.evaluableConfig.bytecode, config.evaluableConfig.constants, entrypoints
            );
            sI9rDispatch = LibEncodedDispatch.encode(expression, BEFORE_ARB_SOURCE_INDEX, BEFORE_ARB_MAX_OUTPUTS);
        }

        return ICLONEABLE_V2_SUCCESS;
    }

    /// @inheritdoc IOrderBookV3ArbOrderTaker
    function arb(TakeOrdersConfigV2 calldata takeOrders, uint256 minimumSenderOutput)
        external
        payable
        nonReentrant
        onlyNotInitializing
    {
        // Mimic what OB would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert NoOrders();
        }

        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

        // Run the access control dispatch if it is set.
        EncodedDispatch dispatch = sI9rDispatch;
        if (EncodedDispatch.unwrap(dispatch) > 0) {
            (uint256[] memory stack, uint256[] memory kvs) = sI9r.eval(
                sI9rStore,
                DEFAULT_STATE_NAMESPACE,
                dispatch,
                LibContext.build(new uint256[][](0), new SignedContextV1[](0))
            );
            // This can only happen if interpreter is broken.
            if (stack.length > 0) {
                revert NonZeroBeforeArbStack();
            }
            // Persist any state changes from the expression.
            if (kvs.length > 0) {
                sI9rStore.set(DEFAULT_STATE_NAMESPACE, kvs);
            }
        }

        IERC20(ordersInputToken).safeApprove(address(sOrderBook), 0);
        IERC20(ordersInputToken).safeApprove(address(sOrderBook), type(uint256).max);
        (uint256 totalInput, uint256 totalOutput) = sOrderBook.takeOrders(takeOrders);
        (totalInput, totalOutput);
        IERC20(ordersInputToken).safeApprove(address(sOrderBook), 0);

        // Send all unspent input tokens to the sender.
        uint256 inputBalance = IERC20(ordersInputToken).balanceOf(address(this));
        if (inputBalance < minimumSenderOutput) {
            revert MinimumOutput(minimumSenderOutput, inputBalance);
        }
        if (inputBalance > 0) {
            IERC20(ordersInputToken).safeTransfer(msg.sender, inputBalance);
        }
        // Send all unspent output tokens to the sender.
        uint256 outputBalance = IERC20(ordersOutputToken).balanceOf(address(this));
        if (outputBalance > 0) {
            IERC20(ordersOutputToken).safeTransfer(msg.sender, outputBalance);
        }

        // Send any remaining gas to the sender.
        // Slither false positive here. We want to send everything to the sender
        // because this contract should be empty of all gas and tokens between
        // uses. Anyone who sends tokens or gas to an arb contract without
        // calling `arb` is going to lose their tokens/gas.
        // See https://github.com/crytic/slither/issues/1658
        Address.sendValue(payable(msg.sender), address(this).balance);
    }

    /// @inheritdoc IOrderBookV3OrderTaker
    function onTakeOrders(address, address, uint256, uint256, bytes calldata)
        public
        virtual
        override
        onlyNotInitializing
    {
        if (msg.sender != address(sOrderBook)) {
            revert BadLender(msg.sender);
        }
    }
}
