// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

import "../ierc3156/IERC3156FlashLender.sol";
import {IExpressionDeployerV3, EvaluableV2} from "rain.interpreter.interface/lib/caller/LibEvaluable.sol";
import {
    EvaluableConfigV3,
    IInterpreterCallerV2,
    SignedContextV1
} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";

/// Import unmodified structures from older versions of `IOrderBook`.
import {IO, ClearConfig, ClearStateChange} from "../deprecated/IOrderBookV2.sol";

/// Thrown when take orders is called with no orders.
error NoOrders();

/// Thrown when take orders is called with a zero maximum input.
error ZeroMaximumInput();

/// Config the order owner may provide to define their order. The `msg.sender`
/// that adds an order cannot modify the owner nor bypass the integrity check of
/// the expression deployer that they specify. However they MAY specify a
/// deployer with a corrupt integrity check, so counterparties and clearers MUST
/// check the DISpair of the order and avoid untrusted pairings.
/// @param validInputs As per `validInputs` on the `Order`.
/// @param validOutputs As per `validOutputs` on the `Order`.
/// @param evaluableConfig Standard `EvaluableConfig` used to produce the
/// `Evaluable` on the order.
/// @param meta Arbitrary bytes that will NOT be used in the order evaluation
/// but MUST be emitted as a Rain `MetaV1` when the order is placed so can be
/// used by offchain processes.
struct OrderConfigV2 {
    IO[] validInputs;
    IO[] validOutputs;
    EvaluableConfigV3 evaluableConfig;
    bytes meta;
}

/// Config for an individual take order from the overall list of orders in a
/// call to `takeOrders`.
/// @param order The order being taken this iteration.
/// @param inputIOIndex The index of the input token in `order` to match with the
/// take order output.
/// @param outputIOIndex The index of the output token in `order` to match with
/// the take order input.
/// @param signedContext Optional additional signed context relevant to the
/// taken order.
struct TakeOrderConfigV2 {
    OrderV2 order;
    uint256 inputIOIndex;
    uint256 outputIOIndex;
    SignedContextV1[] signedContext;
}

/// Defines a fully deployed order ready to evaluate by Orderbook. Identical to
/// `Order` except for the newer `EvaluableV2`.
/// @param owner The owner of the order is the `msg.sender` that added the order.
/// @param handleIO true if there is a "handle IO" entrypoint to run. If false
/// the order book MAY skip calling the interpreter to save gas.
/// @param evaluable Standard `EvaluableV2` with entrypoints for both
/// "calculate order" and "handle IO". The latter MAY be empty bytes, in which
/// case it will be skipped at runtime to save gas.
/// @param validInputs A list of input tokens that are economically equivalent
/// for the purpose of processing this order. Inputs are relative to the order
/// so these tokens will be sent to the owners vault.
/// @param validOutputs A list of output tokens that are economically equivalent
/// for the purpose of processing this order. Outputs are relative to the order
/// so these tokens will be sent from the owners vault.
struct OrderV2 {
    address owner;
    bool handleIO;
    EvaluableV2 evaluable;
    IO[] validInputs;
    IO[] validOutputs;
}

/// Config for a list of orders to take sequentially as part of a `takeOrders`
/// call.
/// @param minimumInput Minimum input from the perspective of the order taker.
/// @param maximumInput Maximum input from the perspective of the order taker.
/// @param maximumIORatio Maximum IO ratio as calculated by the order being
/// taken. The input is from the perspective of the order so higher ratio means
/// worse deal for the order taker.
/// @param orders Ordered list of orders that will be taken until the limit is
/// hit. Takers are expected to prioritise orders that appear to be offering
/// better deals i.e. lower IO ratios. This prioritisation and sorting MUST
/// happen offchain, e.g. via. some simulator.
/// @param data If nonzero length, triggers `onTakeOrders` on the caller of
/// `takeOrders` with this data. This allows the caller to perform arbitrary
/// onchain actions between receiving their input tokens, before having to send
/// their output tokens.
struct TakeOrdersConfigV2 {
    uint256 minimumInput;
    uint256 maximumInput;
    uint256 maximumIORatio;
    TakeOrderConfigV2[] orders;
    bytes data;
}

/// @title IOrderBookV3
/// @notice An orderbook that deploys _strategies_ represented as interpreter
/// expressions rather than individual orders. The order book contract itself
/// behaves similarly to an `ERC4626` vault but with much more fine grained
/// control over how tokens are allocated and moved internally by their owners,
/// and without any concept of "shares". Token owners MAY deposit and withdraw
/// their tokens under arbitrary vault IDs on a per-token basis, then define
/// orders that specify how tokens move between vaults according to an expression.
/// The expression returns a maximum amount and a token input/output ratio from
/// the perpective of the order. When two expressions intersect, as in their
/// ratios are the inverse of each other, then tokens can move between vaults.
///
/// For example, consider order A with input TKNA and output TKNB with a constant
/// ratio of 100:1. This order in isolation has no ability to move tokens. If
/// an order B appears with input TKNB and output TKNA and a ratio of 1:100 then
/// this is a perfect match with order A. In this case 100 TKNA will move from
/// order B to order A and 1 TKNB will move from order A to order B.
///
/// IO ratios are always specified as input:output and are 18 decimal fixed point
/// values. The maximum amount that can be moved in the current clearance is also
/// set by the order expression as an 18 decimal fixed point value.
///
/// Typically orders will not clear when their match is exactly 1:1 as the
/// clearer needs to pay gas to process the match. Each order will get exactly
/// the ratio it calculates when it does clear so if there is _overlap_ in the
/// ratios then the clearer keeps the difference. In our above example, consider
/// order B asking a ratio of 1:110 instead of 1:100. In this case 100 TKNA will
/// move from order B to order A and 10 TKNA will move to the clearer's vault and
/// 1 TKNB will move from order A to order B. In the case of fixed prices this is
/// not very interesting as order B could more simply take order A directly for
/// cheaper rather than involving a third party. Indeed, Orderbook supports a
/// direct "take orders" method that works similar to a "market buy". In the case
/// of dynamic expression based ratios, it allows both order A and order B to
/// clear non-interactively according to their strategy, trading off active
/// management, dealing with front-running, MEV, etc. for zero-gas and
/// exact-ratio clearance.
///
/// The general invariant for clearing and take orders is:
///
/// ```
/// ratioA = InputA / OutputA
/// ratioB = InputB / OutputB
/// ratioA * ratioB = ( InputA * InputB ) / ( OutputA * OutputB )
/// OutputA >= InputB
/// OutputB >= InputA
///
/// ∴ ratioA * ratioB <= 1
/// ```
///
/// Orderbook is `IERC3156FlashLender` compliant with a 0 fee flash loan
/// implementation to allow external liquidity from other onchain DEXes to match
/// against orderbook expressions. All deposited tokens across all vaults are
/// available for flashloan, the flashloan MAY BE REPAID BY CALLING TAKE ORDER
/// such that Orderbook's liability to its vaults is decreased by an incoming
/// trade from the flashloan borrower. See `ZeroExOrderBookFlashBorrower` for
/// an example of how this works in practise.
///
/// Orderbook supports many to many input/output token relationship, for example
/// some order can specify an array of stables it would be willing to accept in
/// return for some ETH. This removes the need for a combinatorial explosion of
/// order strategies between like assets but introduces the issue of token
/// decimal handling. End users understand that "one" USDT is roughly equal to
/// "one" DAI, but onchain this is incorrect by _12 orders of magnitude_. This
/// is because "one" DAI is `1e18` tokens and "one" USDT is `1e6` tokens. The
/// orderbook is allowing orders to deploy expressions that define _economic
/// equivalence_ but this doesn't map 1:1 with numeric equivalence in a many to
/// many setup behind token decimal convensions. The solution is to require that
/// end users who place orders provide the decimals of each token they include
/// in their valid IO lists, and to calculate all amounts and ratios in their
/// expressions _as though they were 18 decimal fixed point values_. Orderbook
/// will then automatically rescale the expression values before applying the
/// final vault movements. If an order provides the "wrong" decimal values for
/// some token then it will simply calculate its own ratios and amounts
/// incorrectly which will either lead to no matching orders or a very bad trade
/// for the order owner. There is no way that misrepresenting decimals can attack
/// some other order by a counterparty. Orderbook DOES NOT read decimals from
/// tokens onchain because A. this would be gas for an external call to a cold
/// token contract and B. the ERC20 standard specifically states NOT to read
/// decimals from the interface onchain.
///
/// Token amounts and ratios returned by calculate order MUST be 18 decimal fixed
/// point values. Token amounts input to handle IO MUST be the exact absolute
/// values that move between the vaults, i.e. NOT rescaled to 18 decimals. The
/// author of the handle IO expression MUST use the token decimals and amounts to
/// rescale themselves if they want that logic, notably the expression author
/// will need to specify the desired rounding behaviour in the rescaling process.
///
/// When two orders clear there are NO TOKEN MOVEMENTS, only internal vault
/// balances are updated from the input and output vaults. Typically this results
/// in less gas per clear than calling external token transfers and also avoids
/// issues with reentrancy, allowances, external balances etc. This also means
/// that REBASING TOKENS AND TOKENS WITH DYNAMIC BALANCE ARE NOT SUPPORTED.
/// Orderbook ONLY WORKS IF TOKEN BALANCES ARE 1:1 WITH ADDITION/SUBTRACTION PER
/// VAULT MOVEMENT.
///
/// Dust due to rounding errors always favours the order. Output max is rounded
/// down and IO ratios are rounded up. Input and output amounts are always
/// converted to absolute values before applying to vault balances such that
/// orderbook always retains fully collateralised inventory of underlying token
/// balances to support withdrawals, with the caveat that dynamic token balanes
/// are not supported.
///
/// When an order clears it is NOT removed. Orders remain active until the owner
/// deactivates them. This is gas efficient as order owners MAY deposit more
/// tokens in a vault with an order against it many times and the order strategy
/// will continue to be clearable according to its expression. As vault IDs are
/// `uint256` values there are effectively infinite possible vaults for any token
/// so there is no limit to how many active orders any address can have at one
/// time. This also allows orders to be daisy chained arbitrarily where output
/// vaults for some order are the input vaults for some other order.
///
/// Expression storage is namespaced by order owner, so gets and sets are unique
/// to each onchain address. Order owners MUST TAKE CARE not to override their
/// storage sets globally across all their orders, which they can do most simply
/// by hashing the order hash into their get/set keys inside the expression. This
/// gives maximum flexibility for shared state across orders without allowing
/// order owners to attack and overwrite values stored by orders placed by their
/// counterparty.
///
/// Note that each order specifies its own interpreter and deployer so the
/// owner is responsible for not corrupting their own calculations with bad
/// interpreters. This also means the Orderbook MUST assume the interpreter, and
/// notably the interpreter's store, is malicious and guard against reentrancy
/// etc.
///
/// As Orderbook supports any expression that can run on any `IInterpreterV1` and
/// counterparties are available to the order, order strategies are free to
/// implement KYC/membership, tracking, distributions, stock, buybacks, etc. etc.
///
/// Main differences between `IOrderBookV2` and `IOderBookV3`:
/// - Most structs are now primitives to save gas.
/// - Order hash is `bytes32`.
/// - `deposit` and `withdraw` MUST revert if the amount is zero.
/// - adding an order MUST revert if there is no calculation entrypoint.
/// - adding an order MUST revert if there is no handle IO entrypoint.
/// - adding an order MUST revert if there are no inputs.
/// - adding an order MUST revert if there are no outputs.
/// - adding and removing orders MUST return a boolean indicating if the state
/// changed.
/// - new `orderExists` method.
interface IOrderBookV3 is IERC3156FlashLender, IInterpreterCallerV2 {
    /// MUST be thrown by `deposit` if the amount is zero.
    /// @param sender `msg.sender` depositing tokens.
    /// @param token The token being deposited.
    /// @param vaultId The vault ID the tokens are being deposited under.
    error ZeroDepositAmount(address sender, address token, uint256 vaultId);

    /// MUST be thrown by `withdraw` if the amount _requested_ to withdraw is
    /// zero. The withdrawal MAY still not move any tokens if the vault balance
    /// is zero, or the withdrawal is used to repay a flash loan.
    /// @param sender `msg.sender` withdrawing tokens.
    /// @param token The token being withdrawn.
    /// @param vaultId The vault ID the tokens are being withdrawn from.
    error ZeroWithdrawTargetAmount(address sender, address token, uint256 vaultId);

    /// MUST be thrown by `addOrder` if the order has no associated calculation.
    error OrderNoSources();

    /// MUST be thrown by `addOrder` if the order has no associated handle IO.
    error OrderNoHandleIO();

    /// MUST be thrown by `addOrder` if the order has no inputs.
    error OrderNoInputs();

    /// MUST be thrown by `addOrder` if the order has no outputs.
    error OrderNoOutputs();

    /// Some tokens have been deposited to a vault.
    /// @param sender `msg.sender` depositing tokens. Delegated deposits are NOT
    /// supported.
    /// @param token The token being deposited.
    /// @param vaultId The vault ID the tokens are being deposited under.
    /// @param amount The amount of tokens deposited.
    event Deposit(address sender, address token, uint256 vaultId, uint256 amount);

    /// Some tokens have been withdrawn from a vault.
    /// @param sender `msg.sender` withdrawing tokens. Delegated withdrawals are
    /// NOT supported.
    /// @param token The token being withdrawn.
    /// @param vaultId The vault ID the tokens are being withdrawn from.
    /// @param targetAmount The amount of tokens requested to withdraw.
    /// @param amount The amount of tokens withdrawn, can be less than the
    /// target amount if the vault does not have the funds available to cover
    /// the target amount. For example an active order might move tokens before
    /// the withdraw completes.
    event Withdraw(address sender, address token, uint256 vaultId, uint256 targetAmount, uint256 amount);

    /// An order has been added to the orderbook. The order is permanently and
    /// always active according to its expression until/unless it is removed.
    /// @param sender `msg.sender` adding the order and is owner of the order.
    /// @param expressionDeployer The expression deployer that ran the integrity
    /// check for this order. This is NOT included in the `Order` itself but is
    /// important for offchain processes to ignore untrusted deployers before
    /// interacting with them.
    /// @param order The newly added order. MUST be handed back as-is when
    /// clearing orders and contains derived information in addition to the order
    /// config that was provided by the order owner.
    /// @param orderHash The hash of the order as it is recorded onchain. Only
    /// the hash is stored in Orderbook storage to avoid paying gas to store the
    /// entire order.
    event AddOrder(address sender, IExpressionDeployerV3 expressionDeployer, OrderV2 order, bytes32 orderHash);

    /// An order has been removed from the orderbook. This effectively
    /// deactivates it. Orders can be added again after removal.
    /// @param sender `msg.sender` removing the order and is owner of the order.
    /// @param order The removed order.
    /// @param orderHash The hash of the removed order.
    event RemoveOrder(address sender, OrderV2 order, bytes32 orderHash);

    /// Some order has been taken by `msg.sender`. This is the same as them
    /// placing inverse orders then immediately clearing them all, but costs less
    /// gas and is more convenient and reliable. Analogous to a market buy
    /// against the specified orders. Each order that is matched within a the
    /// `takeOrders` loop emits its own individual event.
    /// @param sender `msg.sender` taking the orders.
    /// @param config All config defining the orders to attempt to take.
    /// @param input The input amount from the perspective of sender.
    /// @param output The output amount from the perspective of sender.
    event TakeOrder(address sender, TakeOrderConfigV2 config, uint256 input, uint256 output);

    /// Emitted when attempting to match an order that either never existed or
    /// was removed. An event rather than an error so that we allow attempting
    /// many orders in a loop and NOT rollback on "best effort" basis to clear.
    /// @param sender `msg.sender` clearing the order that wasn't found.
    /// @param owner Owner of the order that was not found.
    /// @param orderHash Hash of the order that was not found.
    event OrderNotFound(address sender, address owner, bytes32 orderHash);

    /// Emitted when an order evaluates to a zero amount. An event rather than an
    /// error so that we allow attempting many orders in a loop and NOT rollback
    /// on a "best effort" basis to clear.
    /// @param sender `msg.sender` clearing the order that had a 0 amount.
    /// @param owner Owner of the order that evaluated to a 0 amount.
    /// @param orderHash Hash of the order that evaluated to a 0 amount.
    event OrderZeroAmount(address sender, address owner, bytes32 orderHash);

    /// Emitted when an order evaluates to a ratio exceeding the counterparty's
    /// maximum limit. An error rather than an error so that we allow attempting
    /// many orders in a loop and NOT rollback on a "best effort" basis to clear.
    /// @param sender `msg.sender` clearing the order that had an excess ratio.
    /// @param owner Owner of the order that had an excess ratio.
    /// @param orderHash Hash of the order that had an excess ratio.
    event OrderExceedsMaxRatio(address sender, address owner, bytes32 orderHash);

    /// Emitted before two orders clear. Covers both orders and includes all the
    /// state before anything is calculated.
    /// @param sender `msg.sender` clearing both orders.
    /// @param alice One of the orders.
    /// @param bob The other order.
    /// @param clearConfig Additional config required to process the clearance.
    event Clear(address sender, OrderV2 alice, OrderV2 bob, ClearConfig clearConfig);

    /// Emitted after two orders clear. Includes all final state changes in the
    /// vault balances, including the clearer's vaults.
    /// @param sender `msg.sender` clearing the order.
    /// @param clearStateChange The final vault state changes from the clearance.
    event AfterClear(address sender, ClearStateChange clearStateChange);

    /// Get the current balance of a vault for a given owner, token and vault ID.
    /// @param owner The owner of the vault.
    /// @param token The token the vault is for.
    /// @param id The vault ID to read.
    /// @return balance The current balance of the vault.
    function vaultBalance(address owner, address token, uint256 id) external view returns (uint256 balance);

    /// `msg.sender` deposits tokens according to config. The config specifies
    /// the vault to deposit tokens under. Delegated depositing is NOT supported.
    /// Depositing DOES NOT mint shares (unlike ERC4626) so the overall vaulted
    /// experience is much simpler as there is always a 1:1 relationship between
    /// deposited assets and vault balances globally and individually. This
    /// mitigates rounding/dust issues, speculative behaviour on derived assets,
    /// possible regulatory issues re: whether a vault share is a security, code
    /// bloat on the vault, complex mint/deposit/withdraw/redeem 4-way logic,
    /// the need for preview functions, etc. etc.
    ///
    /// At the same time, allowing vault IDs to be specified by the depositor
    /// allows much more granular and direct control over token movements within
    /// Orderbook than either ERC4626 vault shares or mere contract-level ERC20
    /// allowances can facilitate.
    //
    /// Vault IDs are namespaced by the token address so there is no risk of
    /// collision between tokens. For example, vault ID 0 for token A is
    /// completely different to vault ID 0 for token B.
    ///
    /// `0` amount deposits are unsupported as underlying token contracts
    /// handle `0` value transfers differently and this would be a source of
    /// confusion. The order book MUST revert with `ZeroDepositAmount` if the
    /// amount is zero.
    ///
    /// @param token The token to deposit.
    /// @param vaultId The vault ID to deposit under.
    /// @param amount The amount of tokens to deposit.
    function deposit(address token, uint256 vaultId, uint256 amount) external;

    /// Allows the sender to withdraw any tokens from their own vaults. If the
    /// withrawer has an active flash loan debt denominated in the same token
    /// being withdrawn then Orderbook will merely reduce the debt and NOT send
    /// the amount of tokens repaid to the flashloan debt.
    ///
    /// MUST revert if the amount _requested_ to withdraw is zero. The withdrawal
    /// MAY still not move any tokens (without revert) if the vault balance is
    /// zero, or the withdrawal is used to repay a flash loan, or due to any
    /// other internal accounting.
    ///
    /// @param token The token to withdraw.
    /// @param vaultId The vault ID to withdraw from.
    /// @param targetAmount The amount of tokens to attempt to withdraw. MAY
    /// result in fewer tokens withdrawn if the vault balance is lower than the
    /// target amount. MAY NOT be zero, the order book MUST revert with
    /// `ZeroWithdrawTargetAmount` if the amount is zero.
    function withdraw(address token, uint256 vaultId, uint256 targetAmount) external;

    /// Given an order config, deploys the expression and builds the full `Order`
    /// for the config, then records it as an active order. Delegated adding an
    /// order is NOT supported. The `msg.sender` that adds an order is ALWAYS
    /// the owner and all resulting vault movements are their own.
    ///
    /// MUST revert with `OrderNoSources` if the order has no associated
    /// calculation and `OrderNoHandleIO` if the order has no handle IO
    /// entrypoint. The calculation MUST return at least two values from
    /// evaluation, the maximum amount and the IO ratio. The handle IO entrypoint
    /// SHOULD return zero values from evaluation. Either MAY revert during
    /// evaluation on the interpreter, which MUST prevent the order from
    /// clearing.
    ///
    /// MUST revert with `OrderNoInputs` if the order has no inputs.
    /// MUST revert with `OrderNoOutputs` if the order has no outputs.
    ///
    /// If the order already exists, the order book MUST NOT change state, which
    /// includes not emitting an event. Instead it MUST return false. If the
    /// order book modifies state it MUST emit an `AddOrder` event and return
    /// true.
    ///
    /// @param config All config required to build an `Order`.
    /// @return stateChanged True if the order was added, false if it already
    /// existed.
    function addOrder(OrderConfigV2 calldata config) external returns (bool stateChanged);

    /// Returns true if the order exists, false otherwise.
    /// @param orderHash The hash of the order to check.
    /// @return exists True if the order exists, false otherwise.
    function orderExists(bytes32 orderHash) external view returns (bool exists);

    /// Order owner can remove their own orders. Delegated order removal is NOT
    /// supported and will revert. Removing an order multiple times or removing
    /// an order that never existed are valid, the event will be emitted and the
    /// transaction will complete with that order hash definitely, redundantly
    /// not live.
    /// @param order The `Order` data exactly as it was added.
    /// @return stateChanged True if the order was removed, false if it did not
    /// exist.
    function removeOrder(OrderV2 calldata order) external returns (bool stateChanged);

    /// Allows `msg.sender` to attempt to fill a list of orders in sequence
    /// without needing to place their own order and clear them. This works like
    /// a market buy but against a specific set of orders. Every order will
    /// looped over and calculated individually then filled maximally until the
    /// request input is reached for the `msg.sender`. The `msg.sender` is
    /// responsible for selecting the best orders at the time according to their
    /// criteria and MAY specify a maximum IO ratio to guard against an order
    /// spiking the ratio beyond what the `msg.sender` expected and is
    /// comfortable with. As orders may be removed and calculate their ratios
    /// dynamically, all issues fulfilling an order other than misconfiguration
    /// by the `msg.sender` are no-ops and DO NOT revert the transaction. This
    /// allows the `msg.sender` to optimistically provide a list of orders that
    /// they aren't sure will completely fill at a good price, and fallback to
    /// more reliable orders further down their list. Misconfiguration such as
    /// token mismatches are errors that revert as this is known and static at
    /// all times to the `msg.sender` so MUST be provided correctly. `msg.sender`
    /// MAY specify a minimum input that MUST be reached across all orders in the
    /// list, otherwise the transaction will revert, this MAY be set to zero.
    ///
    /// Exactly like withdraw, if there is an active flash loan for `msg.sender`
    /// they will have their outstanding loan reduced by the final input amount
    /// preferentially before sending any tokens. Notably this allows arb bots
    /// implemented as flash loan borrowers to connect orders against external
    /// liquidity directly by paying back the loan with a `takeOrders` call and
    /// outputting the result of the external trade.
    ///
    /// Rounding errors always favour the order never the `msg.sender`.
    ///
    /// @param config The constraints and list of orders to take, orders are
    /// processed sequentially in order as provided, there is NO ATTEMPT onchain
    /// to predict/filter/sort these orders other than evaluating them as
    /// provided. Inputs and outputs are from the perspective of `msg.sender`
    /// except for values specified by the orders themselves which are the from
    /// the perspective of that order.
    /// @return totalInput Total tokens sent to `msg.sender`, taken from order
    /// vaults processed.
    /// @return totalOutput Total tokens taken from `msg.sender` and distributed
    /// between vaults.
    function takeOrders(TakeOrdersConfigV2 calldata config)
        external
        returns (uint256 totalInput, uint256 totalOutput);

    /// Allows `msg.sender` to match two live orders placed earlier by
    /// non-interactive parties and claim a bounty in the process. The clearer is
    /// free to select any two live orders on the order book for matching and as
    /// long as they have compatible tokens, ratios and amounts, the orders will
    /// clear. Clearing the orders DOES NOT remove them from the orderbook, they
    /// remain live until explicitly removed by their owner. Even if the input
    /// vault balances are completely emptied, the orders remain live until
    /// removed. This allows order owners to deploy a strategy over a long period
    /// of time and periodically top up the input vaults. Clearing two orders
    /// from the same owner is disallowed.
    ///
    /// Any mismatch in the ratios between the two orders will cause either more
    /// inputs than there are available outputs (transaction will revert) or less
    /// inputs than there are available outputs. In the latter case the excess
    /// outputs are given to the `msg.sender` of clear, to the vaults they
    /// specify in the clear config. This not only incentivises "automatic" clear
    /// calls for both alice and bob, but incentivises _prioritising greater
    /// ratio differences_ with a larger bounty. The second point is important
    /// because it implicitly prioritises orders that are further from the
    /// current market price, thus putting constant increasing pressure on the
    /// entire system the further it drifts from the norm, no matter how esoteric
    /// the individual order expressions and sizings might be.
    ///
    /// All else equal there are several factors that would impact how reliably
    /// some order clears relative to the wider market, such as:
    ///
    /// - Bounties are effectively percentages of cleared amounts so larger
    ///   orders have larger bounties and cover gas costs more easily
    /// - High gas on the network means that orders are harder to clear
    ///   profitably so the negative spread of the ratios will need to be larger
    /// - Complex and stateful expressions cost more gas to evalulate so the
    ///   negative spread will need to be larger
    /// - Erratic behavior of the order owner could reduce the willingness of
    ///   third parties to interact if it could result in wasted gas due to
    ///   orders suddently being removed before clearance etc.
    /// - Dynamic and highly volatile words used in the expression could be
    ///   ignored or low priority by clearers who want to be sure that they can
    ///   accurately predict the ratios that they include in their clearance
    /// - Geopolitical issues such as sanctions and regulatory restrictions could
    ///   cause issues for certain owners and clearers
    ///
    /// @param alice Some order to clear.
    /// @param bob Another order to clear.
    /// @param clearConfig Additional configuration for the clearance such as
    /// how to handle the bounty payment for the `msg.sender`.
    /// @param aliceSignedContext Optional signed context that is relevant to A.
    /// @param bobSignedContext Optional signed context that is relevant to B.
    function clear(
        OrderV2 memory alice,
        OrderV2 memory bob,
        ClearConfig calldata clearConfig,
        SignedContextV1[] memory aliceSignedContext,
        SignedContextV1[] memory bobSignedContext
    ) external;
}
