// SPDX-License-Identifier: CAL
pragma solidity =0.8.18;

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {Multicall} from "openzeppelin-contracts/contracts/utils/Multicall.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";

import "rain.math.fixedpoint/FixedPointDecimalArithmeticOpenZeppelin.sol";
import "rain.math.fixedpoint/FixedPointDecimalScale.sol";
import "rain.interpreter/lib/LibEncodedDispatch.sol";
import "rain.interpreter/lib/LibContext.sol";
import {
    DeployerDiscoverableMetaV1,
    DeployerDiscoverableMetaV1ConstructionConfig,
    LibMeta
} from "rain.interpreter/abstract/DeployerDiscoverableMetaV1.sol";

import "../interface/unstable/IOrderBookV3.sol";
import "../lib/LibOrder.sol";
import "../lib/LibOrderBook.sol";
import "../abstract/OrderBookFlashLender.sol";

/// Thrown when the `msg.sender` modifying an order is not its owner.
/// @param sender `msg.sender` attempting to modify the order.
/// @param owner The owner of the order.
error NotOrderOwner(address sender, address owner);

/// Thrown when the input and output tokens don't match, in either direction.
/// @param aliceToken The input or output of one order.
/// @param bobToken The input or output of the other order that doesn't match a.
error TokenMismatch(address aliceToken, address bobToken);

/// Thrown when the minimum input is not met.
/// @param minimumInput The minimum input required.
/// @param input The input that was achieved.
error MinimumInput(uint256 minimumInput, uint256 input);

/// Thrown when two orders have the same owner during clear.
/// @param owner The owner of both orders.
error SameOwner(address owner);

/// @dev Hash of the caller contract metadata for construction.
bytes32 constant CALLER_META_HASH = bytes32(0xa72b1351656daad5908b906f15ad318da93554430d296150444616ae0be4c92f);

/// @dev Value that signifies that an order is live in the internal mapping.
/// Anything nonzero is equally useful.
uint256 constant LIVE_ORDER = 1;

/// @dev Value that signifies that an order is dead in the internal mapping.
uint256 constant DEAD_ORDER = 0;

/// @dev Entrypoint to a calculate the amount and ratio of an order.
SourceIndex constant CALCULATE_ORDER_ENTRYPOINT = SourceIndex.wrap(0);
/// @dev Entrypoint to handle the final internal vault movements resulting from
/// matching multiple calculated orders.
SourceIndex constant HANDLE_IO_ENTRYPOINT = SourceIndex.wrap(1);

/// @dev Minimum outputs for calculate order are the amount and ratio.
uint256 constant CALCULATE_ORDER_MIN_OUTPUTS = 2;
/// @dev Maximum outputs for calculate order are the amount and ratio.
uint16 constant CALCULATE_ORDER_MAX_OUTPUTS = 2;

/// @dev Handle IO has no outputs as it only responds to vault movements.
uint256 constant HANDLE_IO_MIN_OUTPUTS = 0;
/// @dev Handle IO has no outputs as it only response to vault movements.
uint16 constant HANDLE_IO_MAX_OUTPUTS = 0;

/// @dev Orderbook context is actually fairly complex. The calling context column
/// is populated before calculate order, but the remaining columns are only
/// available to handle IO as they depend on the full evaluation of calculuate
/// order, and cross referencing against the same from the counterparty, as well
/// as accounting limits such as current vault balances, etc.
/// The token address and decimals for vault inputs and outputs IS available to
/// the calculate order entrypoint, but not the final vault balances/diff.
uint256 constant CALLING_CONTEXT_COLUMNS = 4;
/// @dev Base context from LibContext.
uint256 constant CONTEXT_BASE_COLUMN = 0;

/// @dev Contextual data available to both calculate order and handle IO. The
/// order hash, order owner and order counterparty. IMPORTANT NOTE that the
/// typical base context of an order with the caller will often be an unrelated
/// clearer of the order rather than the owner or counterparty.
uint256 constant CONTEXT_CALLING_CONTEXT_COLUMN = 1;
/// @dev Calculations column contains the DECIMAL RESCALED calculations but
/// otherwise provided as-is according to calculate order entrypoint
uint256 constant CONTEXT_CALCULATIONS_COLUMN = 2;
/// @dev Vault inputs are the literal token amounts and vault balances before and
/// after for the input token from the perspective of the order. MAY be
/// significantly different to the calculated amount due to insufficient vault
/// balances from either the owner or counterparty, etc.
uint256 constant CONTEXT_VAULT_INPUTS_COLUMN = 3;
/// @dev Vault outputs are the same as vault inputs but for the output token from
/// the perspective of the order.
uint256 constant CONTEXT_VAULT_OUTPUTS_COLUMN = 4;

/// @dev Row of the token address for vault inputs and outputs columns.
uint256 constant CONTEXT_VAULT_IO_TOKEN = 0;
/// @dev Row of the token decimals for vault inputs and outputs columns.
uint256 constant CONTEXT_VAULT_IO_TOKEN_DECIMALS = 1;
/// @dev Row of the vault ID for vault inputs and outputs columns.
uint256 constant CONTEXT_VAULT_IO_VAULT_ID = 2;
/// @dev Row of the vault balance before the order was cleared for vault inputs
/// and outputs columns.
uint256 constant CONTEXT_VAULT_IO_BALANCE_BEFORE = 3;
/// @dev Row of the vault balance difference after the order was cleared for
/// vault inputs and outputs columns. The diff is ALWAYS POSITIVE as it is a
/// `uint256` so it must be added to input balances and subtraced from output
/// balances.
uint256 constant CONTEXT_VAULT_IO_BALANCE_DIFF = 4;
/// @dev Length of a vault IO column.
uint256 constant CONTEXT_VAULT_IO_ROWS = 5;

/// @title OrderBook
/// See `IOrderBookV1` for more documentation.
contract OrderBook is IOrderBookV3, ReentrancyGuard, Multicall, OrderBookFlashLender, DeployerDiscoverableMetaV1 {
    using Math for uint256;
    using LibUint256Array for uint256[];
    using SafeERC20 for IERC20;
    using FixedPointDecimalArithmeticOpenZeppelin for uint256;
    using FixedPointDecimalScale for uint256;
    using LibOrder for Order;
    using LibUint256Array for uint256;

    /// All hashes of all active orders. There's nothing interesting in the value
    /// it's just nonzero if the order is live. The key is the hash of the order.
    /// Removing an order sets the value back to zero so it is identical to the
    /// order never existing and gives a gas refund on removal.
    /// The order hash includes its owner so there's no need to build a multi
    /// level mapping, each order hash MUST uniquely identify the order globally.
    /// order hash => order is live
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(uint256 => uint256) internal sOrders;

    /// @dev Vault balances are stored in a mapping of owner => token => vault ID
    /// This gives 1:1 parity with the `IOrderBookV1` interface but keeping the
    /// `sFoo` naming convention for storage variables.
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(address => mapping(address => mapping(uint256 => uint256))) internal sVaultBalances;

    /// Initializes the orderbook upon construction for compatibility with
    /// Open Zeppelin upgradeable contracts. Orderbook itself does NOT support
    /// factory deployments as each order is a unique expression deployment
    /// rather than needing to wrap up expressions with proxies.
    constructor(DeployerDiscoverableMetaV1ConstructionConfig memory config)
        DeployerDiscoverableMetaV1(CALLER_META_HASH, config)
    {}

    /// @inheritdoc IOrderBookV3
    function vaultBalance(address owner, address token, uint256 vaultId) external view override returns (uint256) {
        return sVaultBalances[owner][token][vaultId];
    }

    /// @inheritdoc IOrderBookV3
    function deposit(address token, uint256 vaultId, uint256 amount) external nonReentrant {
        if (amount == 0) {
            revert ZeroDepositAmount(msg.sender, token, vaultId);
        }
        // It is safest with vault deposits to move tokens in to the Orderbook
        // before updating internal vault balances although we have a reentrancy
        // guard in place anyway.
        emit Deposit(msg.sender, token, vaultId, amount);
        //slither-disable-next-line reentrancy-benign
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        sVaultBalances[msg.sender][token][vaultId] += amount;
    }

    /// @inheritdoc IOrderBookV3
    function withdraw(address token, uint256 vaultId, uint256 targetAmount) external nonReentrant {
        if (targetAmount == 0) {
            revert ZeroWithdrawTargetAmount(msg.sender, token, vaultId);
        }
        uint256 currentVaultBalance = sVaultBalances[msg.sender][token][vaultId];
        uint256 withdrawAmount = targetAmount.min(currentVaultBalance);
        // The overflow check here is redundant with .min above, so technically
        // this is overly conservative but we REALLY don't want withdrawals to
        // exceed vault balances.
        sVaultBalances[msg.sender][token][vaultId] = currentVaultBalance - withdrawAmount;
        emit Withdraw(msg.sender, token, vaultId, targetAmount, withdrawAmount);
        _decreaseFlashDebtThenSendToken(token, msg.sender, withdrawAmount);
    }

    /// @inheritdoc IOrderBookV3
    function addOrder(OrderConfig calldata config) external nonReentrant {
        (IInterpreterV1 interpreter, IInterpreterStoreV1 store, address expression) = config
            .evaluableConfig
            .deployer
            .deployExpression(
            config.evaluableConfig.sources,
            config.evaluableConfig.constants,
            LibUint256Array.arrayFrom(CALCULATE_ORDER_MIN_OUTPUTS, HANDLE_IO_MIN_OUTPUTS)
        );
        Order memory order = Order(
            msg.sender,
            config.evaluableConfig.sources[SourceIndex.unwrap(HANDLE_IO_ENTRYPOINT)].length > 0,
            Evaluable(interpreter, store, expression),
            config.validInputs,
            config.validOutputs
        );
        uint256 orderHash = order.hash();

        // Check that the order is not already live. As the order hash includes
        // the expression address, this can only happen if the deployer returns
        // the same expression address for multiple deployments. This is
        // technically possible but not something we want to support.
        if (sOrders[orderHash] != DEAD_ORDER) {
            revert OrderExists(msg.sender, orderHash);
        }
        //slither-disable-next-line reentrancy-benign
        sOrders[orderHash] = LIVE_ORDER;
        emit AddOrder(msg.sender, config.evaluableConfig.deployer, order, orderHash);

        if (config.meta.length > 0) {
            LibMeta.checkMetaUnhashed(config.meta);
            emit MetaV1(msg.sender, orderHash, config.meta);
        }
    }

    function _calculateOrderDispatch(address expression_) internal pure returns (EncodedDispatch) {
        return LibEncodedDispatch.encode(expression_, CALCULATE_ORDER_ENTRYPOINT, CALCULATE_ORDER_MAX_OUTPUTS);
    }

    function _handleIODispatch(address expression_) internal pure returns (EncodedDispatch) {
        return LibEncodedDispatch.encode(expression_, HANDLE_IO_ENTRYPOINT, HANDLE_IO_MAX_OUTPUTS);
    }

    /// @inheritdoc IOrderBookV3
    function removeOrder(Order calldata order_) external nonReentrant {
        if (msg.sender != order_.owner) {
            revert NotOrderOwner(msg.sender, order_.owner);
        }
        uint256 orderHash_ = order_.hash();
        delete (sOrders[orderHash_]);
        emit RemoveOrder(msg.sender, order_, orderHash_);
    }

    /// @inheritdoc IOrderBookV3
    function takeOrders(TakeOrdersConfig calldata takeOrders_)
        external
        nonReentrant
        returns (uint256 totalInput_, uint256 totalOutput_)
    {
        uint256 i_ = 0;
        TakeOrderConfig memory takeOrder_;
        Order memory order_;
        uint256 remainingInput_ = takeOrders_.maximumInput;
        while (i_ < takeOrders_.orders.length && remainingInput_ > 0) {
            takeOrder_ = takeOrders_.orders[i_];
            order_ = takeOrder_.order;
            uint256 orderHash_ = order_.hash();
            if (sOrders[orderHash_] == DEAD_ORDER) {
                emit OrderNotFound(msg.sender, order_.owner, orderHash_);
            } else {
                if (order_.validInputs[takeOrder_.inputIOIndex].token != takeOrders_.output) {
                    revert TokenMismatch(order_.validInputs[takeOrder_.inputIOIndex].token, takeOrders_.output);
                }
                if (order_.validOutputs[takeOrder_.outputIOIndex].token != takeOrders_.input) {
                    revert TokenMismatch(order_.validOutputs[takeOrder_.outputIOIndex].token, takeOrders_.input);
                }

                OrderIOCalculation memory orderIOCalculation_ = _calculateOrderIO(
                    order_, takeOrder_.inputIOIndex, takeOrder_.outputIOIndex, msg.sender, takeOrder_.signedContext
                );

                // Skip orders that are too expensive rather than revert as we have
                // no way of knowing if a specific order becomes too expensive
                // between submitting to mempool and execution, but other orders may
                // be valid so we want to take advantage of those if possible.
                if (orderIOCalculation_.IORatio > takeOrders_.maximumIORatio) {
                    emit OrderExceedsMaxRatio(msg.sender, order_.owner, orderHash_);
                } else if (orderIOCalculation_.outputMax == 0) {
                    emit OrderZeroAmount(msg.sender, order_.owner, orderHash_);
                } else {
                    // Don't exceed the maximum total input.
                    uint256 input_ = remainingInput_.min(orderIOCalculation_.outputMax);
                    // Always round IO calculations up.
                    uint256 output_ = input_.fixedPointMul(orderIOCalculation_.IORatio, Math.Rounding.Up);

                    remainingInput_ -= input_;
                    totalOutput_ += output_;

                    _recordVaultIO(order_, output_, input_, orderIOCalculation_);
                    emit TakeOrder(msg.sender, takeOrder_, input_, output_);
                }
            }

            unchecked {
                i_++;
            }
        }
        totalInput_ = takeOrders_.maximumInput - remainingInput_;

        if (totalInput_ < takeOrders_.minimumInput) {
            revert MinimumInput(takeOrders_.minimumInput, totalInput_);
        }

        // We already updated vault balances before we took tokens from
        // `msg.sender` which is usually NOT the correct order of operations for
        // depositing to a vault. We rely on reentrancy guards to make this safe.
        IERC20(takeOrders_.output).safeTransferFrom(msg.sender, address(this), totalOutput_);
        // Prioritise paying down any active flash loans before sending any
        // tokens to `msg.sender`.
        _decreaseFlashDebtThenSendToken(takeOrders_.input, msg.sender, totalInput_);
    }

    /// @inheritdoc IOrderBookV3
    function clear(
        Order memory alice_,
        Order memory bob_,
        ClearConfig calldata clearConfig_,
        SignedContextV1[] memory aliceSignedContext_,
        SignedContextV1[] memory bobSignedContext_
    ) external nonReentrant {
        {
            if (alice_.owner == bob_.owner) {
                revert SameOwner(alice_.owner);
            }
            if (
                alice_.validOutputs[clearConfig_.aliceOutputIOIndex].token
                    != bob_.validInputs[clearConfig_.bobInputIOIndex].token
            ) {
                revert TokenMismatch(
                    alice_.validOutputs[clearConfig_.aliceOutputIOIndex].token,
                    bob_.validInputs[clearConfig_.bobInputIOIndex].token
                );
            }

            if (
                bob_.validOutputs[clearConfig_.bobOutputIOIndex].token
                    != alice_.validInputs[clearConfig_.aliceInputIOIndex].token
            ) {
                revert TokenMismatch(
                    alice_.validInputs[clearConfig_.aliceInputIOIndex].token,
                    bob_.validOutputs[clearConfig_.bobOutputIOIndex].token
                );
            }

            // If either order is dead the clear is a no-op other than emitting
            // `OrderNotFound`. Returning rather than erroring makes it easier to
            // bulk clear using `Multicall`.
            if (sOrders[alice_.hash()] == DEAD_ORDER) {
                emit OrderNotFound(msg.sender, alice_.owner, alice_.hash());
                return;
            }
            if (sOrders[bob_.hash()] == DEAD_ORDER) {
                emit OrderNotFound(msg.sender, bob_.owner, bob_.hash());
                return;
            }

            // Emit the Clear event before `eval`.
            emit Clear(msg.sender, alice_, bob_, clearConfig_);
        }
        OrderIOCalculation memory aliceOrderIOCalculation_ = _calculateOrderIO(
            alice_, clearConfig_.aliceInputIOIndex, clearConfig_.aliceOutputIOIndex, bob_.owner, bobSignedContext_
        );
        OrderIOCalculation memory bobOrderIOCalculation_ = _calculateOrderIO(
            bob_, clearConfig_.bobInputIOIndex, clearConfig_.bobOutputIOIndex, alice_.owner, aliceSignedContext_
        );
        ClearStateChange memory clearStateChange_ =
            LibOrderBook._clearStateChange(aliceOrderIOCalculation_, bobOrderIOCalculation_);

        _recordVaultIO(alice_, clearStateChange_.aliceInput, clearStateChange_.aliceOutput, aliceOrderIOCalculation_);
        _recordVaultIO(bob_, clearStateChange_.bobInput, clearStateChange_.bobOutput, bobOrderIOCalculation_);

        {
            // At least one of these will overflow due to negative bounties if
            // there is a spread between the orders.
            uint256 aliceBounty_ = clearStateChange_.aliceOutput - clearStateChange_.bobInput;
            uint256 bobBounty_ = clearStateChange_.bobOutput - clearStateChange_.aliceInput;
            if (aliceBounty_ > 0) {
                sVaultBalances[msg.sender][alice_.validOutputs[clearConfig_.aliceOutputIOIndex].token][clearConfig_
                    .aliceBountyVaultId] += aliceBounty_;
            }
            if (bobBounty_ > 0) {
                sVaultBalances[msg.sender][bob_.validOutputs[clearConfig_.bobOutputIOIndex].token][clearConfig_
                    .bobBountyVaultId] += bobBounty_;
            }
        }

        emit AfterClear(msg.sender, clearStateChange_);
    }

    /// Main entrypoint into an order calculates the amount and IO ratio. Both
    /// are always treated as 18 decimal fixed point values and then rescaled
    /// according to the order's definition of each token's actual fixed point
    /// decimals.
    /// @param order The order to evaluate.
    /// @param inputIOIndex The index of the input token being calculated for.
    /// @param outputIOIndex The index of the output token being calculated for.
    /// @param counterparty The counterparty of the order as it is currently
    /// being cleared against.
    /// @param signedContext Any signed context provided by the clearer/taker
    /// that the order may need for its calculations.
    function _calculateOrderIO(
        Order memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        address counterparty,
        SignedContextV1[] memory signedContext
    ) internal view virtual returns (OrderIOCalculation memory) {
        unchecked {
            uint256 orderHash = order.hash();

            uint256[][] memory context;
            {
                uint256[][] memory callingContext = new uint256[][](
                    CALLING_CONTEXT_COLUMNS
                );
                callingContext[CONTEXT_CALLING_CONTEXT_COLUMN - 1] =
                    LibUint256Array.arrayFrom(orderHash, uint256(uint160(order.owner)), uint256(uint160(counterparty)));

                callingContext[CONTEXT_VAULT_INPUTS_COLUMN - 1] = LibUint256Array.arrayFrom(
                    uint256(uint160(order.validInputs[inputIOIndex].token)),
                    order.validInputs[inputIOIndex].decimals,
                    order.validInputs[inputIOIndex].vaultId,
                    sVaultBalances[order.owner][order.validInputs[inputIOIndex].token][order.validInputs[inputIOIndex]
                        .vaultId],
                    // Don't know the balance diff yet!
                    0
                );

                callingContext[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = LibUint256Array.arrayFrom(
                    uint256(uint160(order.validOutputs[outputIOIndex].token)),
                    order.validOutputs[outputIOIndex].decimals,
                    order.validOutputs[outputIOIndex].vaultId,
                    sVaultBalances[order.owner][order.validOutputs[outputIOIndex].token][order.validOutputs[outputIOIndex]
                        .vaultId],
                    // Don't know the balance diff yet!
                    0
                );
                context = LibContext.build(callingContext, signedContext);
            }

            // The state changes produced here are handled in _recordVaultIO so
            // that local storage writes happen before writes on the interpreter.
            StateNamespace namespace = StateNamespace.wrap(uint256(uint160(order.owner)));
            // Slither false positive. External calls within loops are fine if
            // the caller controls which contracts are called as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            (uint256[] memory calculateOrderStack, uint256[] memory calculateOrderKVs) = order
                .evaluable
                .interpreter
                .eval(order.evaluable.store, namespace, _calculateOrderDispatch(order.evaluable.expression), context);

            uint256 orderOutputMax = calculateOrderStack[calculateOrderStack.length - 2];
            uint256 orderIORatio = calculateOrderStack[calculateOrderStack.length - 1];

            // Rescale order output max from 18 FP to whatever decimals the
            // output token is using.
            // Always round order output down.
            orderOutputMax = orderOutputMax.scaleN(
                order.validOutputs[outputIOIndex].decimals,
                // Saturate the order max output because if we were willing to
                // give more than this on a scale up, we should be comfortable
                // giving less.
                // Round DOWN to be conservative and give away less if there's
                // any loss of precision during scale down.
                FLAG_SATURATE
            );
            // Rescale the ratio from 18 FP according to the difference in
            // decimals between input and output.
            // Always round IO ratio up.
            orderIORatio = orderIORatio.scaleRatio(
                order.validOutputs[outputIOIndex].decimals,
                order.validInputs[inputIOIndex].decimals,
                // DO NOT saturate ratios because this would reduce the effective
                // IO ratio, which would mean that saturating would make the deal
                // worse for the order. Instead we overflow, and round up to get
                // the best possible deal.
                FLAG_ROUND_UP
            );

            // The order owner can't send more than the smaller of their vault
            // balance or their per-order limit.
            orderOutputMax = orderOutputMax.min(
                sVaultBalances[order.owner][order.validOutputs[outputIOIndex].token][order.validOutputs[outputIOIndex]
                    .vaultId]
            );

            // Populate the context with the output max rescaled and vault capped
            // and the rescaled ratio.
            context[CONTEXT_CALCULATIONS_COLUMN] = LibUint256Array.arrayFrom(orderOutputMax, orderIORatio);

            return OrderIOCalculation(orderOutputMax, orderIORatio, context, namespace, calculateOrderKVs);
        }
    }

    /// Given an order, final input and output amounts and the IO calculation
    /// verbatim from `_calculateOrderIO`, dispatch the handle IO entrypoint if
    /// it exists and update the order owner's vault balances.
    /// @param order The order that is being cleared.
    /// @param input The exact token input amount to move into the owner's
    /// vault.
    /// @param output The exact token output amount to move out of the owner's
    /// vault.
    /// @param orderIOCalculation The verbatim order IO calculation returned by
    /// `_calculateOrderIO`.
    function _recordVaultIO(
        Order memory order,
        uint256 input,
        uint256 output,
        OrderIOCalculation memory orderIOCalculation
    ) internal virtual {
        orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] = input;
        orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] = output;

        if (input > 0) {
            // IMPORTANT! THIS MATH MUST BE CHECKED TO AVOID OVERFLOW.
            sVaultBalances[order.owner][address(
                uint160(orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN])
            )][orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID]] += input;
        }
        if (output > 0) {
            // IMPORTANT! THIS MATH MUST BE CHECKED TO AVOID UNDERFLOW.
            sVaultBalances[order.owner][address(
                uint160(orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN])
            )][orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID]] -= output;
        }

        // Emit the context only once in its fully populated form rather than two
        // nearly identical emissions of a partial and full context.
        emit Context(msg.sender, orderIOCalculation.context);

        // Apply state changes to the interpreter store after the vault balances
        // are updated, but before we call handle IO. We want handle IO to see
        // a consistent view on sets from calculate IO.
        if (orderIOCalculation.kvs.length > 0) {
            order.evaluable.store.set(orderIOCalculation.namespace, orderIOCalculation.kvs);
        }

        // Only dispatch handle IO entrypoint if it is defined, otherwise it is
        // a waste of gas to hit the interpreter a second time.
        if (order.handleIO) {
            // The handle IO eval is run under the same namespace as the
            // calculate order entrypoint.
            (uint256[] memory handleIOStack, uint256[] memory handleIOKVs) = order.evaluable.interpreter.eval(
                order.evaluable.store,
                orderIOCalculation.namespace,
                _handleIODispatch(order.evaluable.expression),
                orderIOCalculation.context
            );
            // There's nothing to be done with the stack.
            (handleIOStack);
            // Apply state changes to the interpreter store from the handle IO
            // entrypoint.
            if (handleIOKVs.length > 0) {
                order.evaluable.store.set(orderIOCalculation.namespace, handleIOKVs);
            }
        }
    }
}
