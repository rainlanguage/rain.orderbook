// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {Multicall} from "openzeppelin-contracts/contracts/utils/Multicall.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/security/ReentrancyGuard.sol";

import "rain.math.fixedpoint/lib/LibFixedPointDecimalArithmeticOpenZeppelin.sol";
import "rain.math.fixedpoint/lib/LibFixedPointDecimalScale.sol";
import "rain.interpreter/src/lib/caller/LibEncodedDispatch.sol";
import "rain.interpreter/src/lib/caller/LibContext.sol";
import {
    DeployerDiscoverableMetaV2,
    DeployerDiscoverableMetaV2ConstructionConfig,
    LibMeta
} from "rain.interpreter/src/abstract/DeployerDiscoverableMetaV2.sol";
import "rain.interpreter/src/lib/bytecode/LibBytecode.sol";

import "../interface/unstable/IOrderBookV3.sol";
import "../interface/unstable/IOrderBookV3OrderTaker.sol";
import "../lib/LibOrder.sol";
import "../abstract/OrderBookV3FlashLender.sol";

/// This will exist in a future version of Open Zeppelin if their main branch is
/// to be believed.
error ReentrancyGuardReentrantCall();

/// Thrown when the `msg.sender` modifying an order is not its owner.
/// @param sender `msg.sender` attempting to modify the order.
/// @param owner The owner of the order.
error NotOrderOwner(address sender, address owner);

/// Thrown when the input and output tokens don't match, in either direction.
/// @param aliceToken The input or output of one order.
/// @param bobToken The input or output of the other order that doesn't match a.
error TokenMismatch(address aliceToken, address bobToken);

/// Thrown when the input and output token decimals don't match, in either
/// direction.
/// @param aliceTokenDecimals The input or output decimals of one order.
/// @param bobTokenDecimals The input or output decimals of the other order.
error TokenDecimalsMismatch(uint8 aliceTokenDecimals, uint8 bobTokenDecimals);

/// Thrown when the minimum input is not met.
/// @param minimumInput The minimum input required.
/// @param input The input that was achieved.
error MinimumInput(uint256 minimumInput, uint256 input);

/// Thrown when two orders have the same owner during clear.
/// @param owner The owner of both orders.
error SameOwner(address owner);

/// @dev Stored value for a live order. NOT a boolean because storing a boolean
/// is more expensive than storing a uint256.
uint256 constant ORDER_LIVE = 1;

/// @dev Stored value for a dead order. `0` is chosen because it is the default
/// value for a mapping, which means all orders are dead unless explicitly made
/// live.
uint256 constant ORDER_DEAD = 0;

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

/// @dev Hash of the caller contract metadata for construction.
bytes32 constant CALLER_META_HASH = bytes32(0x1317ffd909f4ca1cd6402c7dd02501ba5965a5b8787a9627cde7b5f3f8f6f840);

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
struct OrderIOCalculation {
    Order order;
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
contract OrderBook is IOrderBookV3, ReentrancyGuard, Multicall, OrderBookV3FlashLender, DeployerDiscoverableMetaV2 {
    using LibUint256Array for uint256[];
    using SafeERC20 for IERC20;
    using LibOrder for Order;
    using LibUint256Array for uint256;
    using Math for uint256;
    using LibFixedPointDecimalScale for uint256;
    using LibFixedPointDecimalArithmeticOpenZeppelin for uint256;

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

    /// Initializes the orderbook upon construction for compatibility with
    /// Open Zeppelin upgradeable contracts. Orderbook itself does NOT support
    /// factory deployments as each order is a unique expression deployment
    /// rather than needing to wrap up expressions with proxies.
    constructor(DeployerDiscoverableMetaV2ConstructionConfig memory config)
        DeployerDiscoverableMetaV2(CALLER_META_HASH, config)
    {}

    /// Guard against read-only reentrancy.
    /// https://chainsecurity.com/heartbreaks-curve-lp-oracles/
    modifier nonReentrantView() {
        if (_reentrancyGuardEntered()) {
            revert ReentrancyGuardReentrantCall();
        }
        _;
    }

    /// @inheritdoc IOrderBookV3
    // This has a reentrancy guard on it but Slither doesn't know that because
    // it is a read-only reentrancy due to potential cross function reentrancy.
    // https://github.com/crytic/slither/issues/735#issuecomment-1620704314
    //slither-disable-next-line reentrancy-no-eth
    function vaultBalance(address owner, address token, uint256 vaultId)
        external
        view
        override
        nonReentrantView
        returns (uint256)
    {
        return sVaultBalances[owner][token][vaultId];
    }

    /// @inheritdoc IOrderBookV3
    function orderExists(bytes32 orderHash) external view override nonReentrantView returns (bool) {
        return sOrders[orderHash] == ORDER_LIVE;
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
        // Don't allow withdrawals to exceed the current vault balance.
        uint256 withdrawAmount = targetAmount.min(currentVaultBalance);
        if (withdrawAmount > 0) {
            // The overflow check here is redundant with .min above, so
            // technically this is overly conservative but we REALLY don't want
            // withdrawals to exceed vault balances.
            sVaultBalances[msg.sender][token][vaultId] = currentVaultBalance - withdrawAmount;
            emit Withdraw(msg.sender, token, vaultId, targetAmount, withdrawAmount);
            _decreaseFlashDebtThenSendToken(token, msg.sender, withdrawAmount);
        }
    }

    /// @inheritdoc IOrderBookV3
    function addOrder(OrderConfigV2 calldata config) external nonReentrant returns (bool stateChanged) {
        uint256 sourceCount = LibBytecode.sourceCount(config.evaluableConfig.bytecode);
        if (sourceCount == 0) {
            revert OrderNoSources(msg.sender);
        }
        if (sourceCount == 1) {
            revert OrderNoHandleIO(msg.sender);
        }
        if (config.validInputs.length == 0) {
            revert OrderNoInputs(msg.sender);
        }
        if (config.validOutputs.length == 0) {
            revert OrderNoOutputs(msg.sender);
        }
        (IInterpreterV1 interpreter, IInterpreterStoreV1 store, address expression) = config
            .evaluableConfig
            .deployer
            .deployExpression(
            config.evaluableConfig.bytecode,
            config.evaluableConfig.constants,
            LibUint256Array.arrayFrom(CALCULATE_ORDER_MIN_OUTPUTS, HANDLE_IO_MIN_OUTPUTS)
        );

        // Merge our view on the sender/owner and handle IO emptiness with the
        // config and deployer's view on the `Evaluable` to produce the final
        // order.
        Order memory order = Order(
            msg.sender,
            LibBytecode.sourceOpsLength(config.evaluableConfig.bytecode, SourceIndex.unwrap(HANDLE_IO_ENTRYPOINT)) > 0,
            Evaluable(interpreter, store, expression),
            config.validInputs,
            config.validOutputs
        );
        bytes32 orderHash = order.hash();

        // If the order is not dead we return early without state changes.
        if (sOrders[orderHash] == ORDER_DEAD) {
            stateChanged = true;

            //slither-disable-next-line reentrancy-benign
            sOrders[orderHash] = ORDER_LIVE;
            emit AddOrder(msg.sender, config.evaluableConfig.deployer, order, orderHash);

            // We only emit the meta event if there is meta to emit. We do require
            // that the meta self describes as a Rain meta document.
            if (config.meta.length > 0) {
                LibMeta.checkMetaUnhashed(config.meta);
                emit MetaV1(msg.sender, uint256(orderHash), config.meta);
            }
        }
    }

    /// @inheritdoc IOrderBookV3
    function removeOrder(Order calldata order) external nonReentrant returns (bool stateChanged) {
        if (msg.sender != order.owner) {
            revert NotOrderOwner(msg.sender, order.owner);
        }
        bytes32 orderHash = order.hash();
        if (sOrders[orderHash] == ORDER_LIVE) {
            stateChanged = true;
            sOrders[orderHash] = ORDER_DEAD;
            emit RemoveOrder(msg.sender, order, orderHash);
        }
    }

    /// @inheritdoc IOrderBookV3
    function takeOrders(TakeOrdersConfigV2 calldata config)
        external
        nonReentrant
        returns (uint256 totalTakerInput, uint256 totalTakerOutput)
    {
        if (config.orders.length == 0) {
            revert NoOrders();
        }

        uint256 i = 0;
        TakeOrderConfig memory takeOrderConfig;
        Order memory order;

        uint256 remainingTakerInput = config.maximumInput;
        if (remainingTakerInput == 0) {
            revert ZeroMaximumInput();
        }
        while (i < config.orders.length && remainingTakerInput > 0) {
            takeOrderConfig = config.orders[i];
            order = takeOrderConfig.order;
            // Every order needs the same input token.
            if (
                order.validInputs[takeOrderConfig.inputIOIndex].token
                    != config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token
            ) {
                revert TokenMismatch(
                    order.validInputs[takeOrderConfig.inputIOIndex].token,
                    config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token
                );
            }
            // Every order needs the same output token.
            if (
                order.validOutputs[takeOrderConfig.outputIOIndex].token
                    != config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token
            ) {
                revert TokenMismatch(
                    order.validOutputs[takeOrderConfig.outputIOIndex].token,
                    config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token
                );
            }
            // Every order needs the same input token decimals.
            if (
                order.validInputs[takeOrderConfig.inputIOIndex].decimals
                    != config.orders[0].order.validInputs[config.orders[0].inputIOIndex].decimals
            ) {
                revert TokenDecimalsMismatch(
                    order.validInputs[takeOrderConfig.inputIOIndex].decimals,
                    config.orders[0].order.validInputs[config.orders[0].inputIOIndex].decimals
                );
            }
            // Every order needs the same output token decimals.
            if (
                order.validOutputs[takeOrderConfig.outputIOIndex].decimals
                    != config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].decimals
            ) {
                revert TokenDecimalsMismatch(
                    order.validOutputs[takeOrderConfig.outputIOIndex].decimals,
                    config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].decimals
                );
            }

            bytes32 orderHash = order.hash();
            if (sOrders[orderHash] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, order.owner, orderHash);
            } else {
                OrderIOCalculation memory orderIOCalculation = calculateOrderIO(
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
                if (orderIOCalculation.IORatio > config.maximumIORatio) {
                    emit OrderExceedsMaxRatio(msg.sender, order.owner, orderHash);
                } else if (Output18Amount.unwrap(orderIOCalculation.outputMax) == 0) {
                    emit OrderZeroAmount(msg.sender, order.owner, orderHash);
                } else {
                    uint8 takerInputDecimals = order.validOutputs[takeOrderConfig.outputIOIndex].decimals;
                    // Taker is just "market buying" the order output max.
                    Input18Amount takerInput18 = Input18Amount.wrap(Output18Amount.unwrap(orderIOCalculation.outputMax));
                    // Cap the taker input at the remaining input before
                    // calculating the taker output. Keep everything in 18
                    // decimals at this point, which requires rescaling the
                    // remaining taker input to match.
                    {
                        // Round down and saturate when converting remaining taker input to 18 decimals.
                        Input18Amount remainingTakerInput18 =
                            Input18Amount.wrap(remainingTakerInput.scale18(takerInputDecimals, FLAG_SATURATE));
                        if (Input18Amount.unwrap(takerInput18) > Input18Amount.unwrap(remainingTakerInput18)) {
                            takerInput18 = remainingTakerInput18;
                        }
                    }

                    uint256 takerOutput;
                    {
                        // Always round IO calculations up so the taker pays more.
                        Output18Amount takerOutput18 = Output18Amount.wrap(
                            // Use the capped taker input to calculate the taker
                            // output.
                            Input18Amount.unwrap(takerInput18).fixedPointMul(
                                orderIOCalculation.IORatio, Math.Rounding.Up
                            )
                        );
                        takerOutput = Output18Amount.unwrap(takerOutput18).scaleN(
                            order.validInputs[takeOrderConfig.inputIOIndex].decimals, FLAG_ROUND_UP
                        );
                    }

                    uint256 takerInput = Input18Amount.unwrap(takerInput18).scaleN(takerInputDecimals, FLAG_SATURATE);

                    remainingTakerInput -= takerInput;
                    totalTakerOutput += takerOutput;

                    recordVaultIO(order, takerOutput, takerInput, orderIOCalculation);
                    emit TakeOrder(msg.sender, takeOrderConfig, takerInput, takerOutput);
                }
            }

            unchecked {
                i++;
            }
        }
        totalTakerInput = config.maximumInput - remainingTakerInput;

        if (totalTakerInput < config.minimumInput) {
            revert MinimumInput(config.minimumInput, totalTakerInput);
        }

        // Prioritise paying down any active flash loans before sending any
        // tokens to `msg.sender`. We send the tokens to `msg.sender` first
        // adopting a similar pattern to Uniswap flash swaps. We call the caller
        // before attempting to pull tokens from them in order to facilitate
        // better integrations with external liquidity sources. This could be
        // done by the caller using flash loans but this callback:
        // - may be simpler for the caller to implement
        // - allows the caller to call `takeOrders` _before_ placing external
        //   trades, which is important if the order logic itself is dependent on
        //   external data (e.g. prices) that could be modified by the caller's
        //   trades.
        uint256 takerInputAmountSent = _decreaseFlashDebtThenSendToken(
            config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token, msg.sender, totalTakerInput
        );
        if (config.data.length > 0) {
            IOrderBookV3OrderTaker(msg.sender).onTakeOrders(
                config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token,
                config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token,
                takerInputAmountSent,
                totalTakerOutput,
                config.data
            );
        }

        if (totalTakerOutput > 0) {
            // We already updated vault balances before we took tokens from
            // `msg.sender` which is usually NOT the correct order of operations for
            // depositing to a vault. We rely on reentrancy guards to make this safe.
            IERC20(config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token).safeTransferFrom(
                msg.sender, address(this), totalTakerOutput
            );
        }
    }

    /// @inheritdoc IOrderBookV3
    function clear(
        Order memory alice,
        Order memory bob,
        ClearConfig calldata clearConfig,
        SignedContextV1[] memory aliceSignedContext,
        SignedContextV1[] memory bobSignedContext
    ) external nonReentrant {
        {
            if (alice.owner == bob.owner) {
                revert SameOwner(alice.owner);
            }
            if (
                alice.validOutputs[clearConfig.aliceOutputIOIndex].token
                    != bob.validInputs[clearConfig.bobInputIOIndex].token
            ) {
                revert TokenMismatch(
                    alice.validOutputs[clearConfig.aliceOutputIOIndex].token,
                    bob.validInputs[clearConfig.bobInputIOIndex].token
                );
            }

            if (
                alice.validOutputs[clearConfig.aliceOutputIOIndex].decimals
                    != bob.validInputs[clearConfig.bobInputIOIndex].decimals
            ) {
                revert TokenDecimalsMismatch(
                    alice.validOutputs[clearConfig.aliceOutputIOIndex].decimals,
                    bob.validInputs[clearConfig.bobInputIOIndex].decimals
                );
            }

            if (
                bob.validOutputs[clearConfig.bobOutputIOIndex].token
                    != alice.validInputs[clearConfig.aliceInputIOIndex].token
            ) {
                revert TokenMismatch(
                    alice.validInputs[clearConfig.aliceInputIOIndex].token,
                    bob.validOutputs[clearConfig.bobOutputIOIndex].token
                );
            }

            if (
                bob.validOutputs[clearConfig.bobOutputIOIndex].decimals
                    != alice.validInputs[clearConfig.aliceInputIOIndex].decimals
            ) {
                revert TokenDecimalsMismatch(
                    alice.validInputs[clearConfig.aliceInputIOIndex].decimals,
                    bob.validOutputs[clearConfig.bobOutputIOIndex].decimals
                );
            }

            // If either order is dead the clear is a no-op other than emitting
            // `OrderNotFound`. Returning rather than erroring makes it easier to
            // bulk clear using `Multicall`.
            if (sOrders[alice.hash()] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, alice.owner, alice.hash());
                return;
            }
            if (sOrders[bob.hash()] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, bob.owner, bob.hash());
                return;
            }

            // Emit the Clear event before `eval`.
            emit Clear(msg.sender, alice, bob, clearConfig);
        }
        OrderIOCalculation memory aliceOrderIOCalculation = calculateOrderIO(
            alice, clearConfig.aliceInputIOIndex, clearConfig.aliceOutputIOIndex, bob.owner, bobSignedContext
        );
        OrderIOCalculation memory bobOrderIOCalculation = calculateOrderIO(
            bob, clearConfig.bobInputIOIndex, clearConfig.bobOutputIOIndex, alice.owner, aliceSignedContext
        );
        ClearStateChange memory clearStateChange =
            calculateClearStateChange(aliceOrderIOCalculation, bobOrderIOCalculation);

        recordVaultIO(alice, clearStateChange.aliceInput, clearStateChange.aliceOutput, aliceOrderIOCalculation);
        recordVaultIO(bob, clearStateChange.bobInput, clearStateChange.bobOutput, bobOrderIOCalculation);

        {
            // At least one of these will overflow due to negative bounties if
            // there is a spread between the orders.
            uint256 aliceBounty = clearStateChange.aliceOutput - clearStateChange.bobInput;
            uint256 bobBounty = clearStateChange.bobOutput - clearStateChange.aliceInput;
            if (aliceBounty > 0) {
                sVaultBalances[msg.sender][alice.validOutputs[clearConfig.aliceOutputIOIndex].token][clearConfig
                    .aliceBountyVaultId] += aliceBounty;
            }
            if (bobBounty > 0) {
                sVaultBalances[msg.sender][bob.validOutputs[clearConfig.bobOutputIOIndex].token][clearConfig
                    .bobBountyVaultId] += bobBounty;
            }
        }

        emit AfterClear(msg.sender, clearStateChange);
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
        Order memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        address counterparty,
        SignedContextV1[] memory signedContext
    ) internal view returns (OrderIOCalculation memory) {
        unchecked {
            bytes32 orderHash = order.hash();

            uint256[][] memory context;
            {
                uint256[][] memory callingContext = new uint256[][](
                    CALLING_CONTEXT_COLUMNS
                );
                callingContext[CONTEXT_CALLING_CONTEXT_COLUMN - 1] = LibUint256Array.arrayFrom(
                    uint256(orderHash), uint256(uint160(order.owner)), uint256(uint160(counterparty))
                );

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
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            (uint256[] memory calculateOrderStack, uint256[] memory calculateOrderKVs) = order
                .evaluable
                .interpreter
                .eval(order.evaluable.store, namespace, _calculateOrderDispatch(order.evaluable.expression), context);

            Output18Amount orderOutputMax18 = Output18Amount.wrap(calculateOrderStack[calculateOrderStack.length - 2]);
            uint256 orderIORatio = calculateOrderStack[calculateOrderStack.length - 1];

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

            return OrderIOCalculation(
                order, outputIOIndex, orderOutputMax18, orderIORatio, context, namespace, calculateOrderKVs
            );
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
    function recordVaultIO(
        Order memory order,
        uint256 input,
        uint256 output,
        OrderIOCalculation memory orderIOCalculation
    ) internal {
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
            // Slither false positive. External calls within loops are fine if
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            order.evaluable.store.set(orderIOCalculation.namespace, orderIOCalculation.kvs);
        }

        // Only dispatch handle IO entrypoint if it is defined, otherwise it is
        // a waste of gas to hit the interpreter a second time.
        if (order.handleIO) {
            // The handle IO eval is run under the same namespace as the
            // calculate order entrypoint.
            // Slither false positive. External calls within loops are fine if
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
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
                // Slither false positive. External calls within loops are fine
                // if the caller controls which orders are eval'd as they can
                // drop failing calls and resubmit a new transaction.
                // https://github.com/crytic/slither/issues/880
                //slither-disable-next-line calls-loop
                order.evaluable.store.set(orderIOCalculation.namespace, handleIOKVs);
            }
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
        OrderIOCalculation memory aliceOrderIOCalculation,
        OrderIOCalculation memory bobOrderIOCalculation
    ) internal pure returns (ClearStateChange memory clearStateChange) {
        // Calculate the clear state change for Alice.
        (clearStateChange.aliceInput, clearStateChange.aliceOutput) =
            calculateClearStateAlice(aliceOrderIOCalculation, bobOrderIOCalculation);

        // Flip alice and bob to calculate bob's output.
        (clearStateChange.bobInput, clearStateChange.bobOutput) =
            calculateClearStateAlice(bobOrderIOCalculation, aliceOrderIOCalculation);
    }

    function calculateClearStateAlice(
        OrderIOCalculation memory aliceOrderIOCalculation,
        OrderIOCalculation memory bobOrderIOCalculation
    ) internal pure returns (uint256 aliceInput, uint256 aliceOutput) {
        // Always round IO calculations up so that the counterparty pays more.
        // This is the max input that bob can afford, given his own IO ratio
        // and maximum spend/output.
        Input18Amount bobInputMax18 = Input18Amount.wrap(
            Output18Amount.unwrap(bobOrderIOCalculation.outputMax).fixedPointMul(
                bobOrderIOCalculation.IORatio, Math.Rounding.Up
            )
        );
        Output18Amount aliceOutputMax18 = aliceOrderIOCalculation.outputMax;
        // Alice's doesn't need to provide more output than bob's max input.
        if (Output18Amount.unwrap(aliceOutputMax18) > Input18Amount.unwrap(bobInputMax18)) {
            aliceOutputMax18 = Output18Amount.wrap(Input18Amount.unwrap(bobInputMax18));
        }
        // Alice's final output is the scaled version of the 18 decimal output,
        // rounded down to benefit Alice.
        aliceOutput = Output18Amount.unwrap(aliceOutputMax18).scaleN(
            aliceOrderIOCalculation.order.validOutputs[aliceOrderIOCalculation.outputIOIndex].decimals, 0
        );

        // Alice's input is her bob-capped output * her IO ratio, rounded up.
        Input18Amount aliceInput18 = Input18Amount.wrap(
            Output18Amount.unwrap(aliceOutputMax18).fixedPointMul(aliceOrderIOCalculation.IORatio, Math.Rounding.Up)
        );
        aliceInput =
        // Use bob's output decimals as alice's input decimals.
        //
        // This is only safe if we have previously checked that the decimals
        // match for alice and bob per token, otherwise bob could manipulate
        // alice's intent.
        Input18Amount.unwrap(aliceInput18).scaleN(
            bobOrderIOCalculation.order.validOutputs[bobOrderIOCalculation.outputIOIndex].decimals, FLAG_ROUND_UP
        );
    }

    function _calculateOrderDispatch(address expression_) internal pure returns (EncodedDispatch) {
        return LibEncodedDispatch.encode(expression_, CALCULATE_ORDER_ENTRYPOINT, CALCULATE_ORDER_MAX_OUTPUTS);
    }

    function _handleIODispatch(address expression_) internal pure returns (EncodedDispatch) {
        return LibEncodedDispatch.encode(expression_, HANDLE_IO_ENTRYPOINT, HANDLE_IO_MAX_OUTPUTS);
    }
}
