// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {Multicall} from "openzeppelin-contracts/contracts/utils/Multicall.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {
    LibEncodedDispatch,
    EncodedDispatch
} from "rain.interpreter.interface/lib/deprecated/caller/LibEncodedDispatch.sol";
import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";
import {
    SourceIndexV2,
    StateNamespace,
    IInterpreterV4,
    StackItem,
    EvalV4
} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";
import {LibUint256Array} from "rain.solmem/lib/LibUint256Array.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/unstable/IInterpreterStoreV3.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/deprecated/IExpressionDeployerV3.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {LibMeta} from "rain.metadata/lib/LibMeta.sol";
import {IMetaV1_2} from "rain.metadata/interface/unstable/IMetaV1_2.sol";
import {LibOrderBook} from "../../lib/LibOrderBook.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {
    LibTOFUTokenDecimals,
    TOFUTokenDecimals,
    TOFUOutcome,
    TokenDecimalsReadFailure
} from "../../lib/LibTOFUTokenDecimals.sol";

import {
    IOrderBookV5,
    NoOrders,
    OrderV4,
    OrderConfigV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV4,
    ClearConfigV2,
    ClearStateChangeV2,
    ZeroMaximumInput,
    SignedContextV1,
    EvaluableV4,
    TaskV2,
    QuoteV2,
    Float
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {IOrderBookV5OrderTaker} from "rain.orderbook.interface/interface/unstable/IOrderBookV5OrderTaker.sol";
import {LibOrder} from "../../lib/LibOrder.sol";
import {
    CALLING_CONTEXT_COLUMNS,
    CONTEXT_CALLING_CONTEXT_COLUMN,
    CONTEXT_CALCULATIONS_COLUMN,
    CONTEXT_VAULT_IO_BALANCE_DIFF,
    CONTEXT_VAULT_IO_TOKEN_DECIMALS,
    CONTEXT_VAULT_INPUTS_COLUMN,
    CONTEXT_VAULT_IO_TOKEN,
    CONTEXT_VAULT_OUTPUTS_COLUMN,
    CONTEXT_VAULT_IO_VAULT_ID
} from "../../lib/LibOrderBook.sol";
import {OrderBookV4FlashLender} from "../../abstract/OrderBookV4FlashLender.sol";
import {LibBytes32Array} from "rain.solmem/lib/LibBytes32Array.sol";
import {LibBytes32Matrix} from "rain.solmem/lib/LibBytes32Matrix.sol";

/// This will exist in a future version of Open Zeppelin if their main branch is
/// to be believed.
error ReentrancyGuardReentrantCall();

/// Thrown when the `msg.sender` modifying an order is not its owner.
/// @param sender `msg.sender` attempting to modify the order.
/// @param owner The owner of the order.
error NotOrderOwner(address sender, address owner);

/// Thrown when the input and output tokens don't match, in either direction.
error TokenMismatch();

/// Thrown when the input token is the output token.
error TokenSelfTrade();

/// Thrown when the input and output token decimals don't match, in either
/// direction.
error TokenDecimalsMismatch();

/// Thrown when the minimum input is not met.
/// @param minimumInput The minimum input required.
/// @param input The input that was achieved.
error MinimumInput(Float minimumInput, Float input);

/// Thrown when two orders have the same owner during clear.
error SameOwner();

/// Thrown when calculate order expression wants inputs.
/// @param inputs The inputs the expression wants.
error UnsupportedCalculateInputs(uint256 inputs);

/// Thrown when calculate order expression offers too few outputs.
/// @param outputs The outputs the expression offers.
error UnsupportedCalculateOutputs(uint256 outputs);

/// Thrown when a negative input is being recorded against vault balances.
error NegativeInput();

/// Thrown when a negative output is being recorded against vault balances.
error NegativeOutput();

/// Thrown when a negative vault balance is being recorded.
/// @param vaultBalance The negative vault balance being recorded.
error NegativeVaultBalance(Float vaultBalance);

/// Thrown when a negative amount is being applied to a vault balance.
/// @param amount The negative amount being applied.
error NegativeVaultBalanceChange(Float amount);

/// Thrown when a negative pull is attempted.
error NegativePull();

/// Thrown when a negative push is attempted.
error NegativePush();

/// Throws when a negative bounty is calculated.
error NegativeBounty();

/// Thrown when clear output amounts are both zero.
error ClearZeroAmount();

/// @dev Stored value for a live order. NOT a boolean because storing a boolean
/// is more expensive than storing a uint256.
uint256 constant ORDER_LIVE = 1;

/// @dev Stored value for a dead order. `0` is chosen because it is the default
/// value for a mapping, which means all orders are dead unless explicitly made
/// live.
uint256 constant ORDER_DEAD = 0;

/// @dev Entrypoint to a calculate the amount and ratio of an order.
SourceIndexV2 constant CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0);
/// @dev Entrypoint to handle the final internal vault movements resulting from
/// matching multiple calculated orders.
SourceIndexV2 constant HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1);

/// @dev Minimum outputs for calculate order are the amount and ratio.
uint256 constant CALCULATE_ORDER_MIN_OUTPUTS = 2;
/// @dev Maximum outputs for calculate order are the amount and ratio.
uint16 constant CALCULATE_ORDER_MAX_OUTPUTS = 2;

/// @dev Handle IO has no outputs as it only responds to vault movements.
uint256 constant HANDLE_IO_MIN_OUTPUTS = 0;
/// @dev Handle IO has no outputs as it only response to vault movements.
uint16 constant HANDLE_IO_MAX_OUTPUTS = 0;

/// All information resulting from an order calculation that allows for vault IO
/// to be calculated and applied, then the handle IO entrypoint to be dispatched.
/// @param outputMax The UNSCALED maximum output calculated by the order
/// expression. WILL BE RESCALED ACCORDING TO TOKEN DECIMALS to an 18 fixed
/// point decimal number for the purpose of calculating actual vault movements.
/// The output max is CAPPED AT THE OUTPUT VAULT BALANCE OF THE ORDER OWNER.
/// The order is guaranteed that the total output of this single clearance cannot
/// exceed this (subject to rescaling). It is up to the order expression to track
/// values over time if the output max is to impose a global limit across many
/// transactions and counterparties.
/// @param IORatio The UNSCALED order ratio as input/output from the perspective
/// of the order. As each counterparty's input is the other's output, the IORatio
/// calculated by each order is inverse of its counterparty. IORatio is SCALED
/// ACCORDING TO TOKEN DECIMALS to allow 18 decimal fixed point math over the
/// vault balances. I.e. `1e18` returned from the expression is ALWAYS "one" as
/// ECONOMIC EQUIVALENCE between two tokens, but this will be rescaled according
/// to the decimals of the token. For example, if DAI and USDT have a ratio of
/// `1e18` then in reality `1e12` DAI will move in the vault for every `1` USDT
/// that moves, because DAI has `1e18` decimals per $1 peg and USDT has `1e6`
/// decimals per $1 peg. THE ORDER DEFINES THE DECIMALS for each token, NOT the
/// token itself, because the token MAY NOT report its decimals as per it being
/// optional in the ERC20 specification.
/// @param context The entire 2D context array, initialized from the context
/// passed into the order calculations and then populated with the order
/// calculations and vault IO before being passed back to handle IO entrypoint.
/// @param namespace The `StateNamespace` to be passed to the store for calculate
/// IO state changes.
/// @param kvs KVs returned from calculate order entrypoint to pass to the store
/// before calling handle IO entrypoint.
struct OrderIOCalculationV4 {
    OrderV4 order;
    uint256 outputIOIndex;
    Float outputMax;
    //solhint-disable-next-line var-name-mixedcase
    Float IORatio;
    bytes32[][] context;
    StateNamespace namespace;
    bytes32[] kvs;
}

type Output18Amount is uint256;

type Input18Amount is uint256;

/// @title OrderBook
/// See `IOrderBookV1` for more documentation.
contract OrderBook is IOrderBookV5, IMetaV1_2, ReentrancyGuard, Multicall, OrderBookV4FlashLender {
    using LibUint256Array for uint256[];
    using SafeERC20 for IERC20;
    using LibOrder for OrderV4;
    using LibUint256Array for uint256;
    using LibDecimalFloat for Float;
    using LibBytes32Array for bytes32;

    /// All hashes of all active orders. There's nothing interesting in the value
    /// it's just nonzero if the order is live. The key is the hash of the order.
    /// Removing an order sets the value back to zero so it is identical to the
    /// order never existing.
    /// The order hash includes its owner so there's no need to build a multi
    /// level mapping, each order hash MUST uniquely identify the order globally.
    /// order hash => order is live
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(bytes32 orderHash => uint256 liveness) internal sOrders;

    mapping(address token => TOFUTokenDecimals tofuTokenDecimals) internal sTOFUTokenDecimals;

    /// @dev Vault balances are stored in a mapping of owner => token => vault ID
    /// This gives 1:1 parity with the `IOrderBookV1` interface but keeping the
    /// `sFoo` naming convention for storage variables.
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(address owner => mapping(address token => mapping(bytes32 vaultId => Float balance))) internal
        sVaultBalances;

    /// @inheritdoc IOrderBookV5
    function vaultBalance2(address owner, address token, bytes32 vaultId) external view override returns (Float) {
        return sVaultBalances[owner][token][vaultId];
    }

    /// @inheritdoc IOrderBookV5
    function orderExists(bytes32 orderHash) external view override returns (bool) {
        return sOrders[orderHash] == ORDER_LIVE;
    }

    /// @inheritdoc IOrderBookV5
    function entask2(TaskV2[] calldata post) external nonReentrant {
        LibOrderBook.doPost(new bytes32[][](0), post);
    }

    /// @inheritdoc IOrderBookV5
    function deposit3(address token, bytes32 vaultId, Float depositAmount, TaskV2[] calldata post)
        external
        nonReentrant
    {
        if (!depositAmount.gt(Float.wrap(0))) {
            revert ZeroDepositAmount(msg.sender, token, vaultId);
        }

        (uint256 depositAmountUint256, uint8 decimals) = pullTokens(token, depositAmount);

        // It is safest with vault deposits to move tokens in to the Orderbook
        // before updating internal vault balances although we have a reentrancy
        // guard in place anyway.
        emit DepositV2(msg.sender, token, vaultId, depositAmountUint256);

        (Float beforeBalance, Float afterBalance) = increaseVaultBalance(msg.sender, token, vaultId, depositAmount);

        if (post.length != 0) {
            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(
                    LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(token))),
                        bytes32(vaultId),
                        Float.unwrap(beforeBalance),
                        Float.unwrap(afterBalance),
                        bytes32(uint256(decimals))
                    )
                ),
                post
            );
        }
    }

    /// @inheritdoc IOrderBookV5
    function withdraw3(address token, bytes32 vaultId, Float targetAmount, TaskV2[] calldata post)
        external
        nonReentrant
    {
        if (!targetAmount.gt(Float.wrap(0))) {
            revert ZeroWithdrawTargetAmount(msg.sender, token, vaultId);
        }

        Float currentVaultBalance = sVaultBalances[msg.sender][token][vaultId];
        Float withdrawAmount = targetAmount.min(currentVaultBalance);

        (Float beforeBalance, Float afterBalance) = decreaseVaultBalance(msg.sender, token, vaultId, withdrawAmount);

        (uint256 withdrawAmountUint256, uint8 decimals) = pushTokens(token, withdrawAmount);

        emit WithdrawV2(msg.sender, token, vaultId, targetAmount, withdrawAmount, withdrawAmountUint256);

        if (post.length != 0) {
            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(
                    LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(token))),
                        vaultId,
                        Float.unwrap(beforeBalance),
                        Float.unwrap(afterBalance),
                        Float.unwrap(targetAmount),
                        bytes32(uint256(decimals))
                    )
                ),
                post
            );
        }
    }

    /// @inheritdoc IOrderBookV5
    function addOrder3(OrderConfigV4 calldata orderConfig, TaskV2[] calldata post)
        external
        nonReentrant
        returns (bool)
    {
        if (orderConfig.validInputs.length == 0) {
            revert OrderNoInputs();
        }
        if (orderConfig.validOutputs.length == 0) {
            revert OrderNoOutputs();
        }

        // Merge our view on the sender/owner and handle IO emptiness with the
        // config and deployer's view on the `EvaluableV2` to produce the final
        // order.
        OrderV4 memory order = OrderV4(
            msg.sender, orderConfig.evaluable, orderConfig.validInputs, orderConfig.validOutputs, orderConfig.nonce
        );
        bytes32 orderHash = order.hash();

        bool stateChange = sOrders[orderHash] == ORDER_DEAD;

        // If the order is not dead we return early without state changes.
        if (stateChange) {
            // This has to come after the external call to deploy the expression
            // because the order hash is derived from the expression and DISPair
            // addresses.
            //slither-disable-next-line reentrancy-benign
            sOrders[orderHash] = ORDER_LIVE;
            emit AddOrderV3(order.owner, orderHash, order);

            // We only emit the meta event if there is meta to emit. We do require
            // that the meta self describes as a Rain meta document.
            if (orderConfig.meta.length > 0) {
                LibMeta.checkMetaUnhashedV1(orderConfig.meta);
                emit MetaV1_2(order.owner, orderHash, orderConfig.meta);
            }

            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(LibBytes32Array.arrayFrom(orderHash, bytes32(uint256(uint160(msg.sender))))),
                post
            );
        }

        return stateChange;
    }

    /// @inheritdoc IOrderBookV5
    function removeOrder3(OrderV4 calldata order, TaskV2[] calldata post)
        external
        nonReentrant
        returns (bool stateChanged)
    {
        if (msg.sender != order.owner) {
            revert NotOrderOwner(msg.sender, order.owner);
        }
        bytes32 orderHash = order.hash();
        if (sOrders[orderHash] == ORDER_LIVE) {
            stateChanged = true;
            sOrders[orderHash] = ORDER_DEAD;
            emit RemoveOrderV3(msg.sender, orderHash, order);

            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(LibBytes32Array.arrayFrom(orderHash, bytes32(uint256(uint160(msg.sender))))),
                post
            );
        }
    }

    /// @inheritdoc IOrderBookV5
    function quote2(QuoteV2 calldata quoteConfig) external view returns (bool, Float, Float) {
        bytes32 orderHash = quoteConfig.order.hash();

        if (sOrders[orderHash] != ORDER_LIVE) {
            return (false, Float.wrap(0), Float.wrap(0));
        }

        if (
            quoteConfig.order.validInputs[quoteConfig.inputIOIndex].token
                == quoteConfig.order.validOutputs[quoteConfig.outputIOIndex].token
        ) {
            revert TokenSelfTrade();
        }

        OrderIOCalculationV4 memory orderIOCalculation = calculateOrderIO(
            quoteConfig.order,
            quoteConfig.inputIOIndex,
            quoteConfig.outputIOIndex,
            msg.sender,
            quoteConfig.signedContext
        );
        return (true, orderIOCalculation.outputMax, orderIOCalculation.IORatio);
    }

    /// @inheritdoc IOrderBookV5
    // Most of the cyclomatic complexity here is due to the error handling within
    // the loop. The actual logic is fairly linear.
    //slither-disable-next-line cyclomatic-complexity
    function takeOrders3(TakeOrdersConfigV4 calldata config)
        external
        nonReentrant
        returns (Float totalTakerInput, Float totalTakerOutput)
    {
        if (config.orders.length == 0) {
            revert NoOrders();
        }

        TakeOrderConfigV4 memory takeOrderConfig;
        OrderV4 memory order;

        // Allocate a region of memory to hold pointers. We don't know how many
        // will run at this point, but we conservatively set aside a slot for
        // every order in case we need it, rather than attempt to dynamically
        // resize the array later. There's no guarantee that a dynamic solution
        // would even be cheaper gas-wise, and it would almost certainly be more
        // complex.
        OrderIOCalculationV4[] memory orderIOCalculationsToHandle;
        {
            uint256 length = config.orders.length;
            assembly ("memory-safe") {
                let ptr := mload(0x40)
                orderIOCalculationsToHandle := ptr
                mstore(0x40, add(ptr, mul(add(length, 1), 0x20)))
            }
        }

        {
            Float remainingTakerInput = config.maximumInput;
            if (!remainingTakerInput.gt(Float.wrap(0))) {
                revert ZeroMaximumInput();
            }

            uint256 i = 0;
            while (i < config.orders.length && remainingTakerInput.gt(Float.wrap(0))) {
                takeOrderConfig = config.orders[i];
                order = takeOrderConfig.order;
                // Every order needs the same input token.
                // Every order needs the same output token.
                if (
                    (
                        order.validInputs[takeOrderConfig.inputIOIndex].token
                            != config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token
                    )
                        || (
                            order.validOutputs[takeOrderConfig.outputIOIndex].token
                                != config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token
                        )
                ) {
                    revert TokenMismatch();
                }

                if (
                    order.validInputs[takeOrderConfig.inputIOIndex].token
                        == order.validOutputs[takeOrderConfig.outputIOIndex].token
                ) {
                    revert TokenSelfTrade();
                }

                bytes32 orderHash = order.hash();
                if (sOrders[orderHash] == ORDER_DEAD) {
                    emit OrderNotFound(msg.sender, order.owner, orderHash);
                } else {
                    OrderIOCalculationV4 memory orderIOCalculation = calculateOrderIO(
                        order,
                        takeOrderConfig.inputIOIndex,
                        takeOrderConfig.outputIOIndex,
                        msg.sender,
                        takeOrderConfig.signedContext
                    );

                    // Skip orders that are too expensive rather than revert as we have
                    // no way of knowing if a specific order becomes too expensive
                    // between submitting to mempool and execution, but other orders may
                    // be valid so we want to take advantage of those if possible.
                    if (orderIOCalculation.IORatio.gt(config.maximumIORatio)) {
                        emit OrderExceedsMaxRatio(msg.sender, order.owner, orderHash);
                    } else if (orderIOCalculation.outputMax.isZero()) {
                        emit OrderZeroAmount(msg.sender, order.owner, orderHash);
                    } else {
                        // Taker is just "market buying" the order output max.
                        // Can't exceed the remaining taker input.
                        Float takerInput = orderIOCalculation.outputMax.min(remainingTakerInput);

                        Float takerOutput = orderIOCalculation.IORatio.mul(takerInput);

                        remainingTakerInput = remainingTakerInput.sub(takerInput);

                        totalTakerOutput = totalTakerOutput.add(takerOutput);
                        totalTakerInput = totalTakerInput.add(takerInput);

                        recordVaultIO(takerOutput, takerInput, orderIOCalculation);
                        emit TakeOrderV3(msg.sender, takeOrderConfig, takerInput, takerOutput);

                        // Add the pointer to the order IO calculation to the array
                        // of order IO calculations to handle. This is
                        // unconditional because conditional behaviour is checked
                        // in `handleIO` and we don't want to duplicate that.
                        assembly ("memory-safe") {
                            // Inc the length by 1.
                            let newLength := add(mload(orderIOCalculationsToHandle), 1)
                            mstore(orderIOCalculationsToHandle, newLength)
                            // Store the pointer to the order IO calculation.
                            mstore(add(orderIOCalculationsToHandle, mul(newLength, 0x20)), orderIOCalculation)
                        }
                    }
                }

                unchecked {
                    i++;
                }
            }
        }

        {
            if (totalTakerInput.lt(config.minimumInput)) {
                revert MinimumInput(config.minimumInput, totalTakerInput);
            }
        }

        // We send the tokens to `msg.sender` first adopting a similar pattern to
        // Uniswap flash swaps. We call the caller before attempting to pull
        // tokens from them in order to facilitate better integrations with
        // external liquidity sources. This could be done by the caller using
        // flash loans but this callback:
        // - may be simpler for the caller to implement
        // - allows the caller to call `takeOrders` _before_ placing external
        //   trades, which is important if the order logic itself is dependent on
        //   external data (e.g. prices) that could be modified by the caller's
        //   trades.

        pushTokens(config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token, totalTakerInput);

        if (config.data.length > 0) {
            IOrderBookV5OrderTaker(msg.sender).onTakeOrders2(
                config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token,
                config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token,
                totalTakerInput,
                totalTakerOutput,
                config.data
            );
        }

        pullTokens(config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token, totalTakerOutput);

        unchecked {
            for (uint256 i = 0; i < orderIOCalculationsToHandle.length; i++) {
                handleIO(orderIOCalculationsToHandle[i]);
            }
        }
    }

    /// @inheritdoc IOrderBookV5
    function clear3(
        OrderV4 memory aliceOrder,
        OrderV4 memory bobOrder,
        ClearConfigV2 calldata clearConfig,
        SignedContextV1[] memory aliceSignedContext,
        SignedContextV1[] memory bobSignedContext
    ) external nonReentrant {
        {
            if (aliceOrder.owner == bobOrder.owner) {
                revert SameOwner();
            }
            if (
                (
                    aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token
                        != bobOrder.validInputs[clearConfig.bobInputIOIndex].token
                )
                    || (
                        bobOrder.validOutputs[clearConfig.bobOutputIOIndex].token
                            != aliceOrder.validInputs[clearConfig.aliceInputIOIndex].token
                    )
            ) {
                revert TokenMismatch();
            }

            if (
                aliceOrder.validInputs[clearConfig.aliceInputIOIndex].token
                    == aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token
            ) {
                revert TokenSelfTrade();
            }

            // If either order is dead the clear is a no-op other than emitting
            // `OrderNotFound`. Returning rather than erroring makes it easier to
            // bulk clear using `Multicall`.
            if (sOrders[aliceOrder.hash()] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, aliceOrder.owner, aliceOrder.hash());
                return;
            }
            if (sOrders[bobOrder.hash()] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, bobOrder.owner, bobOrder.hash());
                return;
            }

            // Emit the Clear event before `eval2`.
            emit ClearV3(msg.sender, aliceOrder, bobOrder, clearConfig);
        }
        OrderIOCalculationV4 memory aliceOrderIOCalculation = calculateOrderIO(
            aliceOrder, clearConfig.aliceInputIOIndex, clearConfig.aliceOutputIOIndex, bobOrder.owner, bobSignedContext
        );
        OrderIOCalculationV4 memory bobOrderIOCalculation = calculateOrderIO(
            bobOrder, clearConfig.bobInputIOIndex, clearConfig.bobOutputIOIndex, aliceOrder.owner, aliceSignedContext
        );
        ClearStateChangeV2 memory clearStateChange =
            calculateClearStateChange(aliceOrderIOCalculation, bobOrderIOCalculation);

        recordVaultIO(clearStateChange.aliceInput, clearStateChange.aliceOutput, aliceOrderIOCalculation);
        recordVaultIO(clearStateChange.bobInput, clearStateChange.bobOutput, bobOrderIOCalculation);

        {
            Float aliceBounty = clearStateChange.aliceOutput.sub(clearStateChange.bobInput);
            Float bobBounty = clearStateChange.bobOutput.sub(clearStateChange.aliceInput);

            // A negative bounty means there is a spread between the orders.
            // This is a critical error because it means the DEX could be
            // exploited if allowed.
            if (aliceBounty.lt(Float.wrap(0)) || bobBounty.lt(Float.wrap(0))) {
                revert NegativeBounty();
            }

            increaseVaultBalance(
                msg.sender,
                aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token,
                clearConfig.aliceBountyVaultId,
                aliceBounty
            );
            increaseVaultBalance(
                msg.sender,
                bobOrder.validOutputs[clearConfig.bobOutputIOIndex].token,
                clearConfig.bobBountyVaultId,
                bobBounty
            );
        }

        emit AfterClearV2(msg.sender, clearStateChange);

        handleIO(aliceOrderIOCalculation);
        handleIO(bobOrderIOCalculation);

        // Do this last so we don't swallow errors from the handle IO.
        if (clearStateChange.aliceOutput.isZero() && clearStateChange.bobOutput.isZero()) {
            revert ClearZeroAmount();
        }
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
    function calculateOrderIO(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        address counterparty,
        SignedContextV1[] memory signedContext
    ) internal view returns (OrderIOCalculationV4 memory) {
        unchecked {
            bytes32 orderHash = order.hash();

            bytes32[][] memory context;
            {
                bytes32[][] memory callingContext = new bytes32[][](CALLING_CONTEXT_COLUMNS);
                callingContext[CONTEXT_CALLING_CONTEXT_COLUMN - 1] = LibBytes32Array.arrayFrom(
                    orderHash, bytes32(uint256(uint160(order.owner))), bytes32(uint256(uint160(counterparty)))
                );

                {
                    (TOFUOutcome inputOutcome, uint8 inputDecimals) = LibTOFUTokenDecimals.decimalsForTokenReadOnly(
                        sTOFUTokenDecimals, order.validInputs[inputIOIndex].token
                    );
                    if (inputOutcome != TOFUOutcome.Consistent && inputOutcome != TOFUOutcome.Initial) {
                        revert TokenDecimalsReadFailure(order.validInputs[inputIOIndex].token, inputOutcome);
                    }

                    Float inputTokenVaultBalance = sVaultBalances[order.owner][order.validInputs[inputIOIndex].token][order
                        .validInputs[inputIOIndex].vaultId];
                    callingContext[CONTEXT_VAULT_INPUTS_COLUMN - 1] = LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(order.validInputs[inputIOIndex].token))),
                        bytes32(uint256(inputDecimals)),
                        order.validInputs[inputIOIndex].vaultId,
                        Float.unwrap(inputTokenVaultBalance),
                        // Don't know the balance diff yet!
                        0
                    );
                }

                {
                    (TOFUOutcome outputOutcome, uint8 outputDecimals) = LibTOFUTokenDecimals.decimalsForTokenReadOnly(
                        sTOFUTokenDecimals, order.validOutputs[outputIOIndex].token
                    );
                    if (outputOutcome != TOFUOutcome.Consistent && outputOutcome != TOFUOutcome.Initial) {
                        revert TokenDecimalsReadFailure(order.validOutputs[outputIOIndex].token, outputOutcome);
                    }

                    Float outputTokenVaultBalance = sVaultBalances[order.owner][order.validOutputs[outputIOIndex].token][order
                        .validOutputs[outputIOIndex].vaultId];
                    callingContext[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(order.validOutputs[outputIOIndex].token))),
                        bytes32(uint256(outputDecimals)),
                        order.validOutputs[outputIOIndex].vaultId,
                        Float.unwrap(outputTokenVaultBalance),
                        // Don't know the balance diff yet!
                        0
                    );
                }

                context = LibContext.build(callingContext, signedContext);
            }

            // The state changes produced here are handled in _recordVaultIO so
            // that local storage writes happen before writes on the interpreter.
            StateNamespace namespace = StateNamespace.wrap(uint256(uint160(order.owner)));
            // Slither false positive. External calls within loops are fine if
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            (StackItem[] memory calculateOrderStack, bytes32[] memory calculateOrderKVs) = order
                .evaluable
                .interpreter
                .eval4(
                EvalV4({
                    store: order.evaluable.store,
                    namespace: LibNamespace.qualifyNamespace(namespace, address(this)),
                    bytecode: order.evaluable.bytecode,
                    sourceIndex: CALCULATE_ORDER_ENTRYPOINT,
                    context: context,
                    inputs: new StackItem[](0),
                    stateOverlay: new bytes32[](0)
                })
            );

            // This is a much clearer error message and overall is more efficient
            // than solidity generic index out of bounds errors.
            if (calculateOrderStack.length < CALCULATE_ORDER_MIN_OUTPUTS) {
                revert UnsupportedCalculateOutputs(calculateOrderStack.length);
            }

            Float orderIORatio;
            Float orderOutputMax;
            assembly ("memory-safe") {
                orderIORatio := mload(add(calculateOrderStack, 0x20))
                orderOutputMax := mload(add(calculateOrderStack, 0x40))
            }

            {
                // The order owner can't send more than the smaller of their vault
                // balance or their per-order limit.
                Float ownerVaultBalance = sVaultBalances[order.owner][order.validOutputs[outputIOIndex].token][order
                    .validOutputs[outputIOIndex].vaultId];
                orderOutputMax = orderOutputMax.min(ownerVaultBalance);
            }

            // Populate the context with the output max rescaled and vault capped.
            context[CONTEXT_CALCULATIONS_COLUMN] =
                LibBytes32Array.arrayFrom(Float.unwrap(orderOutputMax), Float.unwrap(orderIORatio));

            return OrderIOCalculationV4({
                order: order,
                outputIOIndex: outputIOIndex,
                outputMax: orderOutputMax,
                IORatio: orderIORatio,
                context: context,
                namespace: namespace,
                kvs: calculateOrderKVs
            });
        }
    }

    function increaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount)
        internal
        returns (Float, Float)
    {
        if (amount.lt(Float.wrap(0))) {
            revert NegativeVaultBalanceChange(amount);
        }

        Float oldBalance = sVaultBalances[owner][token][vaultId];
        Float newBalance = oldBalance.add(amount);

        // This should never be possible as amount is positive and floats are
        // effectively impossible to overflow, but we check it anyway to be safe.
        if (newBalance.lt(Float.wrap(0))) {
            revert NegativeVaultBalance(newBalance);
        }
        sVaultBalances[owner][token][vaultId] = newBalance;

        return (oldBalance, newBalance);
    }

    function decreaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount)
        internal
        returns (Float, Float)
    {
        if (amount.lt(Float.wrap(0))) {
            revert NegativeVaultBalanceChange(amount);
        }

        Float oldBalance = sVaultBalances[owner][token][vaultId];
        Float newBalance = oldBalance.sub(amount);

        // This can definitely happen, so needs to be guarded against.
        // There's no specific check anywhere else that vault balances don't go
        // negative, so this function should be used everywhere for safety.
        if (newBalance.lt(Float.wrap(0))) {
            revert NegativeVaultBalance(newBalance);
        }
        sVaultBalances[owner][token][vaultId] = newBalance;

        return (oldBalance, newBalance);
    }

    /// Given an order, final input and output amounts and the IO calculation
    /// verbatim from `_calculateOrderIO`, dispatch the handle IO entrypoint if
    /// it exists and update the order owner's vault balances.
    /// @param input The input amount.
    /// @param output The output amount.
    /// @param orderIOCalculation The order IO calculation produced by
    function recordVaultIO(Float input, Float output, OrderIOCalculationV4 memory orderIOCalculation) internal {
        orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] = Float.unwrap(input);
        orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] = Float.unwrap(output);

        increaseVaultBalance(
            orderIOCalculation.order.owner,
            address(uint160(uint256(orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN]))),
            orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID],
            input
        );

        decreaseVaultBalance(
            orderIOCalculation.order.owner,
            address(uint160(uint256(orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN]))),
            orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID],
            output
        );

        // Emit the context only once in its fully populated form rather than two
        // nearly identical emissions of a partial and full context.
        emit ContextV2(msg.sender, orderIOCalculation.context);
    }

    function handleIO(OrderIOCalculationV4 memory orderIOCalculation) internal {
        // Apply state changes to the interpreter store after the vault balances
        // are updated, but before we call handle IO. We want handle IO to see
        // a consistent view on sets from calculate IO.
        if (orderIOCalculation.kvs.length > 0) {
            // Slither false positive. External calls within loops are fine if
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            orderIOCalculation.order.evaluable.store.set(orderIOCalculation.namespace, orderIOCalculation.kvs);
        }

        // The handle IO eval is run under the same namespace as the
        // calculate order entrypoint.
        // Slither false positive. External calls within loops are fine if
        // the caller controls which orders are eval'd as they can drop
        // failing calls and resubmit a new transaction.
        // https://github.com/crytic/slither/issues/880
        //slither-disable-next-line calls-loop
        (StackItem[] memory handleIOStack, bytes32[] memory handleIOKVs) = orderIOCalculation
            .order
            .evaluable
            .interpreter
            .eval4(
            EvalV4({
                store: orderIOCalculation.order.evaluable.store,
                namespace: LibNamespace.qualifyNamespace(orderIOCalculation.namespace, address(this)),
                bytecode: orderIOCalculation.order.evaluable.bytecode,
                sourceIndex: HANDLE_IO_ENTRYPOINT,
                context: orderIOCalculation.context,
                inputs: new StackItem[](0),
                stateOverlay: new bytes32[](0)
            })
        );
        // There's nothing to be done with the stack.
        (handleIOStack);
        // Apply state changes to the interpreter store from the handle IO
        // entrypoint.
        if (handleIOKVs.length > 0) {
            // Slither false positive. External calls within loops are fine
            // if the caller controls which orders are eval'd as they can
            // drop failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            orderIOCalculation.order.evaluable.store.set(orderIOCalculation.namespace, handleIOKVs);
        }
    }

    /// Calculates the clear state change given both order calculations for order
    /// alice and order bob. The input of each is their output multiplied by
    /// their IO ratio and the output of each is the smaller of their maximum
    /// output and the counterparty IO * max output.
    /// @param aliceOrderIOCalculation Order calculation for Alice.
    /// @param bobOrderIOCalculation Order calculation for Bob.
    /// @return clearStateChange The clear state change with absolute inputs and
    /// outputs for Alice and Bob.
    function calculateClearStateChange(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) internal pure returns (ClearStateChangeV2 memory clearStateChange) {
        // Calculate the clear state change for Alice.
        (clearStateChange.aliceInput, clearStateChange.aliceOutput) =
            calculateClearStateAlice(aliceOrderIOCalculation, bobOrderIOCalculation);

        // Flip alice and bob to calculate bob's output.
        (clearStateChange.bobInput, clearStateChange.bobOutput) =
            calculateClearStateAlice(bobOrderIOCalculation, aliceOrderIOCalculation);
    }

    function calculateClearStateAlice(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) internal pure returns (Float aliceInput, Float aliceOutput) {
        // Alice's input is her output * her IO ratio.
        aliceInput = aliceOrderIOCalculation.outputMax.mul(aliceOrderIOCalculation.IORatio);

        aliceOutput = aliceOrderIOCalculation.outputMax;

        // If Alice's input is greater than Bob's max output, Alice's input is
        // capped at Bob's max output.
        if (aliceInput.gt(bobOrderIOCalculation.outputMax)) {
            aliceInput = bobOrderIOCalculation.outputMax;

            // Alice's output is capped at her input / her IO ratio.
            aliceOutput = aliceInput.div(aliceOrderIOCalculation.IORatio);
        }
    }

    function pullTokens(address token, Float amount) internal returns (uint256, uint8) {
        (TOFUOutcome tofuOutcome, uint8 decimals) = LibTOFUTokenDecimals.decimalsForToken(sTOFUTokenDecimals, token);
        if (tofuOutcome != TOFUOutcome.Consistent && tofuOutcome != TOFUOutcome.Initial) {
            revert TokenDecimalsReadFailure(token, tofuOutcome);
        }
        if (amount.lt(Float.wrap(0))) {
            revert NegativePull();
        }

        (uint256 amount18, bool lossless) = LibDecimalFloat.toFixedDecimalLossy(amount, decimals);
        // Round truncation up when pulling.
        if (!lossless) {
            // This needs to be checked math as an overflow would cause tokens
            // to silently not be pulled (wraps to 0).
            ++amount18;
        }
        if (amount18 > 0) {
            IERC20(token).safeTransferFrom(msg.sender, address(this), amount18);
        }
        return (amount18, decimals);
    }

    function pushTokens(address token, Float amountFloat) internal returns (uint256, uint8) {
        (TOFUOutcome tofuOutcome, uint8 decimals) = LibTOFUTokenDecimals.decimalsForToken(sTOFUTokenDecimals, token);
        if (tofuOutcome != TOFUOutcome.Consistent && tofuOutcome != TOFUOutcome.Initial) {
            revert TokenDecimalsReadFailure(token, tofuOutcome);
        }

        if (amountFloat.lt(Float.wrap(0))) {
            revert NegativePush();
        }

        (uint256 amount, bool lossless) = LibDecimalFloat.toFixedDecimalLossy(amountFloat, decimals);
        // Truncate when pushing.
        (lossless);
        if (amount > 0) {
            IERC20(token).safeTransfer(msg.sender, amount);
        }

        return (amount, decimals);
    }
}
