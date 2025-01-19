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
import {SourceIndexV2, StateNamespace, IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";
import {LibUint256Array} from "rain.solmem/lib/LibUint256Array.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/deprecated/IExpressionDeployerV3.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {LibMeta} from "rain.metadata/lib/LibMeta.sol";
import {IMetaV1_2} from "rain.metadata/interface/unstable/IMetaV1_2.sol";
import {LibOrderBook} from "../../lib/LibOrderBook.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

import {
    IOrderBookV4,
    NoOrders,
    OrderV3,
    OrderConfigV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    ClearConfig,
    ClearStateChange,
    ZeroMaximumInput,
    SignedContextV1,
    EvaluableV3,
    TaskV1,
    Quote
} from "rain.orderbook.interface/interface/IOrderBookV4.sol";
import {IOrderBookV4OrderTaker} from "rain.orderbook.interface/interface/IOrderBookV4OrderTaker.sol";
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
error MinimumInput(uint256 minimumInput, uint256 input);

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
struct OrderIOCalculationV2 {
    OrderV3 order;
    uint256 outputIOIndex;
    Output18Amount outputMax;
    //solhint-disable-next-line var-name-mixedcase
    uint256 IORatio;
    uint256[][] context;
    StateNamespace namespace;
    uint256[] kvs;
}

type Output18Amount is uint256;

type Input18Amount is uint256;

/// @title OrderBook
/// See `IOrderBookV1` for more documentation.
contract OrderBook is IOrderBookV4, IMetaV1_2, ReentrancyGuard, Multicall, OrderBookV4FlashLender {
    using LibUint256Array for uint256[];
    using SafeERC20 for IERC20;
    using LibOrder for OrderV3;
    using LibUint256Array for uint256;

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

    /// @dev Vault balances are stored in a mapping of owner => token => vault ID
    /// This gives 1:1 parity with the `IOrderBookV1` interface but keeping the
    /// `sFoo` naming convention for storage variables.
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(address owner => mapping(address token => mapping(uint256 vaultId => uint256 balance))) internal
        sVaultBalances;

    /// @inheritdoc IOrderBookV4
    function vaultBalance(address owner, address token, uint256 vaultId) external view override returns (uint256) {
        return sVaultBalances[owner][token][vaultId];
    }

    /// @inheritdoc IOrderBookV4
    function orderExists(bytes32 orderHash) external view override returns (bool) {
        return sOrders[orderHash] == ORDER_LIVE;
    }

    /// @inheritdoc IOrderBookV4
    function entask(TaskV1[] calldata post) external nonReentrant {
        LibOrderBook.doPost(new uint256[][](0), post);
    }

    /// @inheritdoc IOrderBookV4
    function deposit2(address token, uint256 vaultId, uint256 depositAmount, TaskV1[] calldata post)
        external
        nonReentrant
    {
        if (depositAmount == 0) {
            revert ZeroDepositAmount(msg.sender, token, vaultId);
        }
        // It is safest with vault deposits to move tokens in to the Orderbook
        // before updating internal vault balances although we have a reentrancy
        // guard in place anyway.
        emit Deposit(msg.sender, token, vaultId, depositAmount);
        //slither-disable-next-line reentrancy-benign
        IERC20(token).safeTransferFrom(msg.sender, address(this), depositAmount);
        uint256 currentVaultBalance = sVaultBalances[msg.sender][token][vaultId];
        sVaultBalances[msg.sender][token][vaultId] = currentVaultBalance + depositAmount;

        if (post.length != 0) {
            // This can fail as `decimals` is an OPTIONAL part of the ERC20 standard.
            // It's incredibly common anyway. Please let us know if this actually a
            // problem in practice.
            uint256 tokenDecimals = IERC20Metadata(address(uint160(token))).decimals();
            uint256 currentVaultBalance18 = LibFixedPointDecimalScale.scale18(
                currentVaultBalance,
                tokenDecimals,
                // Error on overflow.
                // Rounding down is the default.
                0
            );
            uint256 depositAmount18 = LibFixedPointDecimalScale.scale18(
                depositAmount,
                tokenDecimals,
                // Error on overflow.
                // Rounding down is the default.
                0
            );
            LibOrderBook.doPost(
                LibUint256Matrix.matrixFrom(
                    LibUint256Array.arrayFrom(uint256(uint160(token)), vaultId, currentVaultBalance18, depositAmount18)
                ),
                post
            );
        }
    }

    /// @inheritdoc IOrderBookV4
    function withdraw2(address token, uint256 vaultId, uint256 targetAmount, TaskV1[] calldata post)
        external
        nonReentrant
    {
        if (targetAmount == 0) {
            revert ZeroWithdrawTargetAmount(msg.sender, token, vaultId);
        }
        uint256 currentVaultBalance = sVaultBalances[msg.sender][token][vaultId];
        // Don't allow withdrawals to exceed the current vault balance.
        uint256 withdrawAmount = targetAmount.min(currentVaultBalance);
        if (withdrawAmount > 0) {
            // The overflow check here is redundant with .min above, so
            // technically this is overly conservative but we REALLY don't want
            // withdrawals to exceed vault balances.
            sVaultBalances[msg.sender][token][vaultId] = currentVaultBalance - withdrawAmount;
            emit Withdraw(msg.sender, token, vaultId, targetAmount, withdrawAmount);
            IERC20(token).safeTransfer(msg.sender, withdrawAmount);

            if (post.length != 0) {
                // This can fail as `decimals` is an OPTIONAL part of the ERC20 standard.
                // It's incredibly common anyway. Please let us know if this actually a
                // problem in practice.
                uint256 tokenDecimals = IERC20Metadata(address(uint160(token))).decimals();

                LibOrderBook.doPost(
                    LibUint256Matrix.matrixFrom(
                        LibUint256Array.arrayFrom(
                            uint256(uint160(token)),
                            vaultId,
                            LibFixedPointDecimalScale.scale18(
                                currentVaultBalance,
                                tokenDecimals,
                                // Error on overflow.
                                // Rounding down is the default.
                                0
                            ),
                            LibFixedPointDecimalScale.scale18(
                                withdrawAmount,
                                tokenDecimals,
                                // Error on overflow.
                                // Rounding down is the default.
                                0
                            ),
                            LibFixedPointDecimalScale.scale18(
                                targetAmount,
                                tokenDecimals,
                                // Error on overflow.
                                // Rounding down is the default.
                                0
                            )
                        )
                    ),
                    post
                );
            }
        }
    }

    /// @inheritdoc IOrderBookV4
    function addOrder2(OrderConfigV3 calldata orderConfig, TaskV1[] calldata post)
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
        OrderV3 memory order = OrderV3(
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
            emit AddOrderV2(order.owner, orderHash, order);

            // We only emit the meta event if there is meta to emit. We do require
            // that the meta self describes as a Rain meta document.
            if (orderConfig.meta.length > 0) {
                LibMeta.checkMetaUnhashedV1(orderConfig.meta);
                emit MetaV1_2(order.owner, orderHash, orderConfig.meta);
            }

            LibOrderBook.doPost(
                LibUint256Matrix.matrixFrom(LibUint256Array.arrayFrom(uint256(orderHash), uint256(uint160(msg.sender)))),
                post
            );
        }

        return stateChange;
    }

    /// @inheritdoc IOrderBookV4
    function removeOrder2(OrderV3 calldata order, TaskV1[] calldata post)
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
            emit RemoveOrderV2(msg.sender, orderHash, order);

            LibOrderBook.doPost(
                LibUint256Matrix.matrixFrom(LibUint256Array.arrayFrom(uint256(orderHash), uint256(uint160(msg.sender)))),
                post
            );
        }
    }

    /// @inheritdoc IOrderBookV4
    function quote(Quote calldata quoteConfig) external view returns (bool, uint256, uint256) {
        bytes32 orderHash = quoteConfig.order.hash();

        if (sOrders[orderHash] != ORDER_LIVE) {
            return (false, 0, 0);
        }

        if (
            quoteConfig.order.validInputs[quoteConfig.inputIOIndex].token
                == quoteConfig.order.validOutputs[quoteConfig.outputIOIndex].token
        ) {
            revert TokenSelfTrade();
        }

        OrderIOCalculationV2 memory orderIOCalculation = calculateOrderIO(
            quoteConfig.order,
            quoteConfig.inputIOIndex,
            quoteConfig.outputIOIndex,
            msg.sender,
            quoteConfig.signedContext
        );
        return (true, Output18Amount.unwrap(orderIOCalculation.outputMax), orderIOCalculation.IORatio);
    }

    /// @inheritdoc IOrderBookV4
    // Most of the cyclomatic complexity here is due to the error handling within
    // the loop. The actual logic is fairly linear.
    //slither-disable-next-line cyclomatic-complexity
    function takeOrders2(TakeOrdersConfigV3 calldata config)
        external
        nonReentrant
        returns (
            int256 totalTakerInputSignedCoefficient,
            int256 totalTakerInputExponent,
            int256 totalTakerOutputSignedCoefficient,
            int256 totalTakerOutputExponent
        )
    {
        if (config.orders.length == 0) {
            revert NoOrders();
        }

        TakeOrderConfigV3 memory takeOrderConfig;
        OrderV3 memory order;

        // Allocate a region of memory to hold pointers. We don't know how many
        // will run at this point, but we conservatively set aside a slot for
        // every order in case we need it, rather than attempt to dynamically
        // resize the array later. There's no guarantee that a dynamic solution
        // would even be cheaper gas-wise, and it would almost certainly be more
        // complex.
        OrderIOCalculationV2[] memory orderIOCalculationsToHandle;
        {
            uint256 length = config.orders.length;
            assembly ("memory-safe") {
                let ptr := mload(0x40)
                orderIOCalculationsToHandle := ptr
                mstore(0x40, add(ptr, mul(add(length, 1), 0x20)))
            }
        }

        {
            uint256 remainingTakerInput = config.maximumInput;
            if (remainingTakerInput == 0) {
                revert ZeroMaximumInput();
            }
            uint256 i = 0;
            while (i < config.orders.length && remainingTakerInput > 0) {
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
                    OrderIOCalculationV2 memory orderIOCalculation = calculateOrderIO(
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
                    if (
                        LibDecimalFloat.gt(
                            orderIOCalculation.IORatioSignedCoefficient,
                            orderIOCalculation.IORatioExponent,
                            config.maximumIORatioSignedCoefficient,
                            config.maximumIORatioExponent
                        )
                    ) {
                        emit OrderExceedsMaxRatio(msg.sender, order.owner, orderHash);
                    } else if (
                        LibDecimalFloat.isZero(
                            orderIOCalculation.outputMaxSignedCoefficient, orderIOCalculation.outputMaxExponent
                        )
                    ) {
                        emit OrderZeroAmount(msg.sender, order.owner, orderHash);
                    } else {
                        // Taker is just "market buying" the order output max.
                        // Can't exceed the remaining taker input.
                        (int256 takerInputSignedCoefficient, int256 takerInputExponent) = LibDecimalFloat.min(
                            orderIOCalculation.outputMaxSignedCoefficient,
                            orderIOCalculation.outputMaxExponent,
                            remainingTakerInputSignedCoefficient,
                            remainingTakerInputExponent
                        );

                        (int256 takerOutputSignedCoefficient, int256 takerOutputExponent) = LibDecimalFloat.mul(
                            orderIOCalculation.IORatioSignedCoefficient,
                            orderIOCalculation.IORatioExponent,
                            takerInputSignedCoefficient,
                            takerInputExponent
                        );

                        (remainingTakerInputSignedCoefficient, remainingTakerInputExponent) = LibDecimalFloat.sub(
                            remainingTakerInputSignedCoefficient,
                            remainingTakerInputExponent,
                            takerInputSignedCoefficient,
                            takerInputExponent
                        );

                        (totalTakerOutputSignedCoefficient, totalTakerOutputExponent) = LibDecimalFloat.add(
                            totalTakerOutputSignedCoefficient,
                            totalTakerOutputExponent,
                            takerOutputSignedCoefficient,
                            takerOutputExponent
                        );

                        recordVaultIO(
                            takerOutputSignedCoefficient,
                            takerOutputExponent,
                            takerInputSignedCoefficient,
                            takerInputExponent,
                            orderIOCalculation
                        );
                        emit TakeOrderV2(
                            msg.sender,
                            takeOrderConfig,
                            LibDecimalFloat.pack(takerInputSignedCoefficient, takerInputExponent),
                            LibDecimalFloat.pack(takerOutputSignedCoefficient, takerOutputExponent)
                        );

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
            totalTakerInput = config.maximumInput - remainingTakerInput;
        }

        {
            (int256 minimumInputSignedCoefficient, int256 minimumInputExponent) =
                LibDecimalFloat.unpack(config.minimumInput);
            if (
                LibDecimalFloat.lt(
                    totalTakerInputSignedCoefficient,
                    totalTakerInputExponent,
                    minimumInputSignedCoefficient,
                    minimumInputExponent
                )
            ) {
                revert MinimumInput(
                    config.minimumInput, LibDecimalFloat.pack(totalTakerInputSignedCoefficient, totalTakerInputExponent)
                );
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

        pushTokens(
            IERC20(config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token),
            totalTakerInputSignedCoefficient,
            totalTakerInputExponent
        );

        if (config.data.length > 0) {
            IOrderBookV5OrderTaker(msg.sender).onTakeOrders2(
                config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token,
                config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token,
                LibDecimalFloat.pack(totalTakerInputSignedCoefficient, totalTakerInputExponent),
                LibDecimalFloat.pack(totalTakerOutputSignedCoefficient, totalTakerOutputExponent),
                config.data
            );
        }

        pullTokens(
            IERC20(config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token),
            totalTakerOutputSignedCoefficient,
            totalTakerOutputExponent
        );

        unchecked {
            for (uint256 i = 0; i < orderIOCalculationsToHandle.length; i++) {
                handleIO(orderIOCalculationsToHandle[i]);
            }
        }
    }

    /// @inheritdoc IOrderBookV4
    function clear2(
        OrderV3 memory aliceOrder,
        OrderV3 memory bobOrder,
        ClearConfig calldata clearConfig,
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

            if (
                (
                    aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].decimals
                        != bobOrder.validInputs[clearConfig.bobInputIOIndex].decimals
                )
                    || (
                        bobOrder.validOutputs[clearConfig.bobOutputIOIndex].decimals
                            != aliceOrder.validInputs[clearConfig.aliceInputIOIndex].decimals
                    )
            ) {
                revert TokenDecimalsMismatch();
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
            emit ClearV2(msg.sender, aliceOrder, bobOrder, clearConfig);
        }
        OrderIOCalculationV2 memory aliceOrderIOCalculation = calculateOrderIO(
            aliceOrder, clearConfig.aliceInputIOIndex, clearConfig.aliceOutputIOIndex, bobOrder.owner, bobSignedContext
        );
        OrderIOCalculationV2 memory bobOrderIOCalculation = calculateOrderIO(
            bobOrder, clearConfig.bobInputIOIndex, clearConfig.bobOutputIOIndex, aliceOrder.owner, aliceSignedContext
        );
        ClearStateChange memory clearStateChange =
            calculateClearStateChange(aliceOrderIOCalculation, bobOrderIOCalculation);

        recordVaultIO(clearStateChange.aliceInput, clearStateChange.aliceOutput, aliceOrderIOCalculation);
        recordVaultIO(clearStateChange.bobInput, clearStateChange.bobOutput, bobOrderIOCalculation);

        {
            // At least one of these will overflow due to negative bounties if
            // there is a spread between the orders.
            uint256 aliceBounty = clearStateChange.aliceOutput - clearStateChange.bobInput;
            uint256 bobBounty = clearStateChange.bobOutput - clearStateChange.aliceInput;
            if (aliceBounty > 0) {
                sVaultBalances[msg.sender][aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token][clearConfig
                    .aliceBountyVaultId] += aliceBounty;
            }
            if (bobBounty > 0) {
                sVaultBalances[msg.sender][bobOrder.validOutputs[clearConfig.bobOutputIOIndex].token][clearConfig
                    .bobBountyVaultId] += bobBounty;
            }
        }

        emit AfterClear(msg.sender, clearStateChange);

        handleIO(aliceOrderIOCalculation);
        handleIO(bobOrderIOCalculation);
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
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        address counterparty,
        SignedContextV1[] memory signedContext
    ) internal view returns (OrderIOCalculationV2 memory) {
        unchecked {
            bytes32 orderHash = order.hash();

            uint256[][] memory context;
            {
                uint256[][] memory callingContext = new uint256[][](CALLING_CONTEXT_COLUMNS);
                callingContext[CONTEXT_CALLING_CONTEXT_COLUMN - 1] = LibUint256Array.arrayFrom(
                    uint256(orderHash), uint256(uint160(order.owner)), uint256(uint160(counterparty))
                );

                {
                    uint256 inputTokenVaultBalance = sVaultBalances[order.owner][order.validInputs[inputIOIndex].token][order
                        .validInputs[inputIOIndex].vaultId];
                    callingContext[CONTEXT_VAULT_INPUTS_COLUMN - 1] = LibUint256Array.arrayFrom(
                        uint256(uint160(order.validInputs[inputIOIndex].token)),
                        order.validInputs[inputIOIndex].decimals * 1e18,
                        order.validInputs[inputIOIndex].vaultId,
                        inputTokenVaultBalance,
                        // Don't know the balance diff yet!
                        0
                    );
                }

                {
                    uint256 outputTokenVaultBalance = sVaultBalances[order.owner][order.validOutputs[outputIOIndex]
                        .token][order.validOutputs[outputIOIndex].vaultId];
                    callingContext[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = LibUint256Array.arrayFrom(
                        uint256(uint160(order.validOutputs[outputIOIndex].token)),
                        order.validOutputs[outputIOIndex].decimals * 1e18,
                        order.validOutputs[outputIOIndex].vaultId,
                        outputTokenVaultBalance,
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
            (uint256[] memory calculateOrderStack, uint256[] memory calculateOrderKVs) = order
                .evaluable
                .interpreter
                .eval3(
                order.evaluable.store,
                LibNamespace.qualifyNamespace(namespace, address(this)),
                order.evaluable.bytecode,
                CALCULATE_ORDER_ENTRYPOINT,
                context,
                new uint256[](0)
            );

            // This is a much clearer error message and overall is more efficient
            // than solidity generic index out of bounds errors.
            if (calculateOrderStack.length < CALCULATE_ORDER_MIN_OUTPUTS) {
                revert UnsupportedCalculateOutputs(calculateOrderStack.length);
            }

            uint256 orderIORatio;
            Output18Amount orderOutputMax18;
            assembly ("memory-safe") {
                orderIORatio := mload(add(calculateOrderStack, 0x20))
                orderOutputMax18 := mload(add(calculateOrderStack, 0x40))
            }

            {
                // The order owner can't send more than the smaller of their vault
                // balance or their per-order limit.
                uint256 ownerVaultBalance = sVaultBalances[order.owner][order.validOutputs[outputIOIndex].token][order
                    .validOutputs[outputIOIndex].vaultId];
                // We round down vault balances and don't saturate because we're
                // dealing with real token amounts here. If rescaling would somehow
                // cause an overflow in a real token amount, that's basically an
                // unsupported token, it implies a very small decimals value with
                // very large token total supply. E.g. 0 decimals with a total supply
                // around 10^60. That's beyond what even Uniswap handles, as they use
                // uint112 values internally for tokens.
                // It's possible that if a token has large decimals, e.g. much more
                // than 18, that the owner vault balance could be rounded down enough
                // to cause significant non-dust amounts to be untradeable. In this
                // case the token is not really supported.
                // In either case, the order owner can still withdraw their vault
                // balances in full, they just can't trade that token effectively.
                Output18Amount ownerVaultBalance18 =
                    Output18Amount.wrap(ownerVaultBalance.scale18(order.validOutputs[outputIOIndex].decimals, 0));
                if (Output18Amount.unwrap(orderOutputMax18) > Output18Amount.unwrap(ownerVaultBalance18)) {
                    orderOutputMax18 = ownerVaultBalance18;
                }
            }

            // Populate the context with the output max rescaled and vault capped.
            context[CONTEXT_CALCULATIONS_COLUMN] =
                LibUint256Array.arrayFrom(Output18Amount.unwrap(orderOutputMax18), orderIORatio);

            return OrderIOCalculationV2({
                order: order,
                outputIOIndex: outputIOIndex,
                outputMax: orderOutputMax18,
                IORatio: orderIORatio,
                context: context,
                namespace: namespace,
                kvs: calculateOrderKVs
            });
        }
    }

    /// Given an order, final input and output amounts and the IO calculation
    /// verbatim from `_calculateOrderIO`, dispatch the handle IO entrypoint if
    /// it exists and update the order owner's vault balances.
    /// @param input The exact token input amount to move into the owner's
    /// vault.
    /// @param output The exact token output amount to move out of the owner's
    /// vault.
    /// @param orderIOCalculation The verbatim order IO calculation returned by
    /// `_calculateOrderIO`.
    function recordVaultIO(
        int256 inputSignedCoefficient,
        int256 inputExponent,
        int256 outputSignedCoefficient,
        int256 outputExponent,
        OrderIOCalculationV2 memory orderIOCalculation
    ) internal {
        orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] =
            LibDecimalFloat.pack(inputSignedCoefficient, inputExponent);
        orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] =
            LibDecimalFloat.pack(outputSignedCoefficient, outputExponent);

        if (LibDecimalFloat.lt(inputSignedCoefficient, inputExponent, 0, 0)) {
            revert NegativeInput();
        }

        if (LibDecimalFloat.lt(outputSignedCoefficient, outputExponent, 0, 0)) {
            revert NegativeOutput();
        }

        if (LibDecimalFloat.gt(inputSignedCoefficient, inputExponent, 0, 0)) {
            (int256 inputVaultBalanceSignedCoefficient, int256 inputVaultBalanceExponent) = LibDecimalFloat.unpack(
                sVaultBalances[orderIOCalculation.order.owner][address(
                    uint160(orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN])
                )][orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID]]
            );

            sVaultBalances[orderIOCalculation.order.owner][address(
                uint160(orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN])
            )][orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID]] = LibDecimalFloat
                .pack(
                LibDecimalFloat.add(
                    inputVaultBalanceSignedCoefficient, inputVaultBalanceExponent, inputSignedCoefficient, inputExponent
                )
            );
        }

        if (LibDecimalFloat.gt(outputSignedCoefficient, outputExponent, 0, 0)) {
            (int256 outputVaultBalanceSignedCoefficient, int256 outputVaultBalanceExponent) = LibDecimalFloat.unpack(
                sVaultBalances[orderIOCalculation.order.owner][address(
                    uint160(orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN])
                )][orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID]]
            );

            sVaultBalances[orderIOCalculation.order.owner][address(
                uint160(orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN])
            )][orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID]] = LibDecimalFloat
                .pack(
                LibDecimalFloat.sub(
                    outputVaultBalanceSignedCoefficient,
                    outputVaultBalanceExponent,
                    outputSignedCoefficient,
                    outputExponent
                )
            );
        }

        // Emit the context only once in its fully populated form rather than two
        // nearly identical emissions of a partial and full context.
        emit Context(msg.sender, orderIOCalculation.context);
    }

    function handleIO(OrderIOCalculationV2 memory orderIOCalculation) internal {
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
        (uint256[] memory handleIOStack, uint256[] memory handleIOKVs) = orderIOCalculation
            .order
            .evaluable
            .interpreter
            .eval3(
            orderIOCalculation.order.evaluable.store,
            LibNamespace.qualifyNamespace(orderIOCalculation.namespace, address(this)),
            orderIOCalculation.order.evaluable.bytecode,
            HANDLE_IO_ENTRYPOINT,
            orderIOCalculation.context,
            new uint256[](0)
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
        OrderIOCalculationV2 memory aliceOrderIOCalculation,
        OrderIOCalculationV2 memory bobOrderIOCalculation
    ) internal pure returns (ClearStateChange memory clearStateChange) {
        // Calculate the clear state change for Alice.
        (clearStateChange.aliceInput, clearStateChange.aliceOutput) =
            calculateClearStateAlice(aliceOrderIOCalculation, bobOrderIOCalculation);

        // Flip alice and bob to calculate bob's output.
        (clearStateChange.bobInput, clearStateChange.bobOutput) =
            calculateClearStateAlice(bobOrderIOCalculation, aliceOrderIOCalculation);
    }

    function calculateClearStateAlice(
        OrderIOCalculationV2 memory aliceOrderIOCalculation,
        OrderIOCalculationV2 memory bobOrderIOCalculation
    ) internal pure returns returns (
        int256 aliceInputSignedCoefficient,
        int256 aliceInputExponent,
        int256 aliceOutputMaxSignedCoefficient,
        int256 aliceOutputMaxExponent
    ) {
        // Alice's input is her output * her IO ratio.
        (aliceInputSignedCoefficient, aliceInputExponent) = LibDecimalFloat.mul(
            aliceOrderIOCalculation.outputMaxSignedCoefficient,
            aliceOrderIOCalculation.outputMaxExponent,
            aliceOrderIOCalculation.IORatioSignedCoefficient,
            aliceOrderIOCalculation.IORatioExponent
        );

        aliceOutputMaxSignedCoefficient = aliceOrderIOCalculation.outputMaxSignedCoefficient;
        aliceOutputMaxExponent = aliceOrderIOCalculation.outputMaxExponent;

        // If Alice's input is greater than Bob's max output, Alice's input is
        // capped at Bob's max output.
        if (LibDecimalFloat.gt(aliceInputSignedCoefficient, aliceInputExponent, bobOrderIOCalculation.outputMaxSignedCoefficient, bobOrderIOCalculation.outputMaxExponent)) {
            aliceInputSignedCoefficient = bobOrderIOCalculation.outputMaxSignedCoefficient;
            aliceInputExponent = bobOrderIOCalculation.outputMaxExponent;

            // Alice's output is capped at her input / her IO ratio.
            (aliceOutputMaxSignedCoefficient, aliceOutputMaxExponent) = LibDecimalFloat.div(
                aliceInputSignedCoefficient,
                aliceInputExponent,
                aliceOrderIOCalculation.IORatioSignedCoefficient,
                aliceOrderIOCalculation.IORatioExponent
            );
        }
    }

    function pullTokens(IERC20 token, uint256 amountSignedCoefficient, uint256 amountExponent) internal {
        uint8 decimals = decimalsForToken(token);
        (uint256 amount, bool lossless) =
            LibDecimalFloat.toFixedDecimalLossy(amountSignedCoefficient, amountExponent, decimals);
        // Round truncation up when pulling.
        if (!lossless) {
            ++amount;
        }
        if (amount > 0) {
            token.safeTransferFrom(msg.sender, address(this), amount);
        }
    }

    function pushTokens(IERC20 token, uint256 amountSignedCoefficient, uint256 amountExponent) internal {
        uint8 decimals = decimalsForToken(token);
        (uint256 amount, bool lossless) =
            LibDecimalFloat.toFixedDecimalLossy(amountSignedCoefficient, amountExponent, decimals);
        // Truncate when pushing.
        (lossless);
        if (amount > 0) {
            token.safeTransfer(msg.sender, amount);
        }
    }

    // @TODO enforce the decimals never change for trades and deposits. If it
    // changes only allow withdrawals.
    function decimalsForToken(IERC20 token) internal view returns (uint8) {
        // @TODO fix the path where decimals does not exist and the return can't
        // deserialize to a uint8, whic will error instead of entering the catch.
        try IERC20Metadata(address(token)).decimals() returns (uint8 decimals) {
            return decimals;
        } catch {
            return 18;
        }
    }
}
