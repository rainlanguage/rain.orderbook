# Triage — audit/2026-03-13-01

## Legend
- **PENDING** — not yet triaged
- **FIXED** — code changed
- **DOCUMENTED** — documentation/comments added
- **DISMISSED** — no action needed (with reason)
- **UPSTREAM** — fix belongs in a dependency/submodule

Where a finding was flagged in multiple passes, the primary finding is listed and duplicates are cross-referenced. Triaging the primary finding applies to all its duplicates.

---

## HIGH

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 1 | A03-2 | P1 | OrderBookV6FlashBorrower.sol | Missing ERC20 approval for flash loan repayment token | PENDING | |
| 2 | A03-P2-3 | P2 | OrderBookV6FlashBorrower.sol | Mock skips token transfers, hiding A03-2 | PENDING | Related to A03-2 |
| 3 | A05-1 | P1 | GenericPoolOrderBookV6ArbOrderTaker.sol | Unlimited approval to arbitrary spender with caller-controlled data | PENDING | |
| 4 | A05-P2-1 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | `onTakeOrders2` is completely untested | PENDING | |
| 5 | A15-1 | P1 | Deploy.sol | Route processor bytecode hash check runs unconditionally | PENDING | Dup: A15-P5-1 |
| 6 | A15-P5-1 | P5 | Deploy.sol | Confirms A15-1 | PENDING | Dup of A15-1 |

## MEDIUM

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 7 | A03-1 | P1 | OrderBookV6FlashBorrower.sol | Missing msg.sender (lender) validation in onFlashLoan | PENDING | |
| 8 | A03-3 | P1 | OrderBookV6FlashBorrower.sol | Flash loan amount computed with wrong token decimals | PENDING | Dup: A03-P2-7 |
| 9 | A05-2 | P1 | GenericPoolOrderBookV6ArbOrderTaker.sol | Arbitrary external call sends entire ETH balance | PENDING | |
| 10 | A06-1 | P1 | GenericPoolOrderBookV6FlashBorrower.sol | Unlimited approval to arbitrary spender with no validation | PENDING | Similar to A05-1 |
| 11 | A03-P2-1 | P2 | OrderBookV6FlashBorrower.sol | onFlashLoan has zero direct test coverage for error paths | PENDING | |
| 12 | A03-P2-2 | P2 | OrderBookV6FlashBorrower.sol | FlashLoanFailed error path is never tested | PENDING | |
| 13 | A03-P2-6 | P2 | OrderBookV6FlashBorrower.sol | WrongTask revert path has no test for flash borrower arb contracts | PENDING | |
| 14 | A05-P2-2 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | No test for fallback() behavior | PENDING | |
| 15 | A07-P2-1 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | onTakeOrders2 override has zero test coverage | PENDING | |
| 16 | A15-P5-2 | P5 | Deploy.sol | vm.envAddress reverts unconditionally for route-processor suite | PENDING | |

## LOW

| # | ID | Pass | File | Title | Status | Notes |
|---|-----|------|------|-------|--------|-------|
| 17 | A02-1 | P1 | OrderBookV6ArbOrderTaker.sol | onTakeOrders2 callback has no access control | PENDING | Similar: A07-1 |
| 18 | A04-2 | P1 | OrderBookV6FlashLender.sol | flashLoan lacks reentrancy guard, nested flash loans | PENDING | Same issue as A08-1 |
| 19 | A05-3 | P1 | GenericPoolOrderBookV6ArbOrderTaker.sol | Non-payable fallback with misleading comment | PENDING | Dups: A06-2, A05-P5-01, A06-P5-01, A07-P5-01, A06-P3-1 |
| 20 | A06-2 | P1 | GenericPoolOrderBookV6FlashBorrower.sol | fallback accepts arbitrary calls without payable | PENDING | Dup of A05-3 pattern |
| 21 | A07-1 | P1 | RouteProcessorOrderBookV6ArbOrderTaker.sol | onTakeOrders2 is public with no access control | PENDING | Similar: A02-1 |
| 22 | A08-1 | P1 | OrderBookV6.sol | flashLoan lacks nonReentrant guard | PENDING | Same issue as A04-2 |
| 23 | A15-2 | P1 | Deploy.sol | No explicit revert on failed create in deployRouter() | PENDING | Dup: A15-P5-3 |
| 24 | A01-P2-1 | P2 | OrderBookV6ArbCommon.sol | No test for WrongTask revert through FlashBorrower path | PENDING | |
| 25 | A01-P2-2 | P2 | OrderBookV6ArbCommon.sol | No direct unit test for constructor iTaskHash assignment | PENDING | |
| 26 | A01-P2-3 | P2 | OrderBookV6ArbCommon.sol | No test for onlyValidTask bypass when iTaskHash == 0 | PENDING | |
| 27 | A02-P2-1 | P2 | OrderBookV6ArbOrderTaker.sol | No test for arb5 reverting on zero orders | PENDING | |
| 28 | A02-P2-2 | P2 | OrderBookV6ArbOrderTaker.sol | No test for reentrancy guard on arb5 | PENDING | |
| 29 | A02-P2-3 | P2 | OrderBookV6ArbOrderTaker.sol | onTakeOrders2 has no direct test | PENDING | |
| 30 | A03-P2-4 | P2 | OrderBookV6FlashBorrower.sol | SwapFailed error declared but never used/tested | PENDING | Dup: A03-P4-2 |
| 31 | A03-P2-5 | P2 | OrderBookV6FlashBorrower.sol | NoOrders revert path has no test through flash borrower | PENDING | |
| 32 | A03-P2-7 | P2 | OrderBookV6FlashBorrower.sol | Flash loan amount wrong decimals not caught by tests | PENDING | Dup of A03-3 |
| 33 | A05-P2-3 | P2 | GenericPoolOrderBookV6ArbOrderTaker.sol | Constructor event emission tested only indirectly | PENDING | |
| 34 | A07-P2-2 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No test for onTakeOrders2 called directly by attacker | PENDING | |
| 35 | A07-P2-3 | P2 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No test for constructor with invalid implementationData | PENDING | |
| 36 | A12-P2-1 | P2 | LibOrderBookArb.sol | NonZeroBeforeArbStack and BadLender errors are dead code | PENDING | Related: A02-P4-3 |
| 37 | A12-P2-2 | P2 | LibOrderBookArb.sol | No test verifies token transfers in finalizeArb | PENDING | |
| 38 | A12-P2-3 | P2 | LibOrderBookArb.sol | No test verifies native gas sent to msg.sender | PENDING | |
| 39 | A12-P2-4 | P2 | LibOrderBookArb.sol | No test exercises finalizeArb with realistic non-zero balances | PENDING | |
| 40 | A15-P2-1 | P2 | Deploy.sol | No unit test for deployRouter() return value on create failure | PENDING | |
| 41 | A15-P2-2 | P2 | Deploy.sol | ROUTE_PROCESSOR_4_BYTECODE_HASH constant never verified | PENDING | |
| 42 | A15-P2-3 | P2 | Deploy.sol | No test for BadRouteProcessor error path | PENDING | |
| 43 | A15-P2-4 | P2 | Deploy.sol | No test coverage for individual suite isolation | PENDING | |
| 44 | A06-P2-GAP1 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for BadInitiator error path | PENDING | |
| 45 | A06-P2-GAP2 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for FlashLoanFailed error path | PENDING | |
| 46 | A06-P2-GAP3 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for WrongTask on GenericPoolOrderBookV6FlashBorrower | PENDING | |
| 47 | A06-P2-GAP4 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for NoOrders revert in arb4 | PENDING | |
| 48 | A06-P2-GAP5 | P2 | GenericPoolOrderBookV6FlashBorrower.sol | No test for _exchange failure propagation | PENDING | |
| 49 | A09-P2-GAP1 | P2 | OrderBookV6SubParser.sol | No tests for deposit word parsing (11 words, 0 tests) | PENDING | |
| 50 | A01-P3-1 | P3 | OrderBookV6ArbCommon.sol | Struct @param name mismatch: tasks vs task | PENDING | Dup: A01-P5-1 |
| 51 | A01-P3-2 | P3 | OrderBookV6ArbCommon.sol | Missing documentation on contract, event, state variable, constructor, modifier | PENDING | |
| 52 | A01-P3-3 | P3 | OrderBookV6ArbCommon.sol | BEFORE_ARB_SOURCE_INDEX doc inaccurately scoped to flash loans | PENDING | |
| 53 | A02-P3-1 | P3 | OrderBookV6ArbOrderTaker.sol | Contract-level NatSpec missing | PENDING | |
| 54 | A02-P3-2 | P3 | OrderBookV6ArbOrderTaker.sol | Typo: "evaluabled" in BEFORE_ARB_SOURCE_INDEX doc | PENDING | |
| 55 | A02-P3-3 | P3 | OrderBookV6ArbOrderTaker.sol | onTakeOrders2 empty body contradicts interface MUST requirement | PENDING | |
| 56 | A03-P3-1 | P3 | OrderBookV6FlashBorrower.sol | Stale @title tag: OrderBookV5FlashBorrower | PENDING | Dup: A03-P4-3 |
| 57 | A03-P3-2 | P3 | OrderBookV6FlashBorrower.sol | Stale interface/contract references in arb4 param docs | PENDING | Dup: A03-P5-2 |
| 58 | A03-P3-3 | P3 | OrderBookV6FlashBorrower.sol | Missing @param for orderBook and task in arb4 | PENDING | |
| 59 | A04-P3-1 | P3 | OrderBookV6FlashLender.sol | Misleading NatSpec on maxFlashLoan | PENDING | Dup: A04-P5-1 |
| 60 | A05-P3-1 | P3 | GenericPoolOrderBookV6ArbOrderTaker.sol | Missing contract-level NatSpec | PENDING | |
| 61 | A06-P3-1 | P3 | GenericPoolOrderBookV6FlashBorrower.sol | Misleading fallback comment | PENDING | Dup of A05-3 |
| 62 | A07-P3-1 | P3 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Missing contract-level NatSpec | PENDING | |
| 63 | A07-P3-2 | P3 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Undocumented lossy conversion and silent discard of precision flag | PENDING | |
| 64 | A08-P3-1 | P3 | OrderBookV6.sol | Stale comment in recordVaultIO contradicts code execution order | PENDING | |
| 65 | A11-P3-1 | P3 | LibOrderBook.sol | Missing NatSpec on doPost | PENDING | |
| 66 | A12-P3-1 | P3 | LibOrderBookArb.sol | Missing NatSpec on finalizeArb | PENDING | |
| 67 | A13-P3-1 | P3 | LibOrderBookSubParser.sol | No NatSpec on any of the 28 SubParser functions | PENDING | |
| 68 | A15-P3-1 | P3 | Deploy.sol | @notice references deprecated "mumbai" testnet | PENDING | |
| 69 | A15-P3-2 | P3 | Deploy.sol | deployRouter() has no documentation | PENDING | |
| 70 | A15-P3-3 | P3 | Deploy.sol | run() has no documentation | PENDING | |
| 71 | A15-P3-4 | P3 | Deploy.sol | Five DEPLOYMENT_SUITE_* constants have no documentation | PENDING | |
| 72 | P3-A09-01 | P3 | OrderBookV6SubParser.sol | buildSubParserWordParsers() has no NatSpec | PENDING | |
| 73 | A01-P4-1 | P4 | OrderBookV6ArbCommon.sol | Multiple unused imports | PENDING | |
| 74 | A02-P4-1 | P4 | OrderBookV6ArbOrderTaker.sol | 10 unused imports | PENDING | |
| 75 | A02-P4-2 | P4 | OrderBookV6ArbOrderTaker.sol | Duplicate constant BEFORE_ARB_SOURCE_INDEX | PENDING | |
| 76 | A02-P4-3 | P4 | OrderBookV6ArbOrderTaker.sol | Dead error NonZeroBeforeArbInputs | PENDING | Related: A12-P2-1 |
| 77 | A03-P4-1 | P4 | OrderBookV6FlashBorrower.sol | 7 unused imports | PENDING | |
| 78 | A03-P4-2 | P4 | OrderBookV6FlashBorrower.sol | Dead error SwapFailed | PENDING | Dup of A03-P2-4 |
| 79 | A03-P4-3 | P4 | OrderBookV6FlashBorrower.sol | Stale NatSpec @title OrderBookV5FlashBorrower | PENDING | Dup of A03-P3-1 |
| 80 | A06-P4-1 | P4 | GenericPoolOrderBookV6FlashBorrower.sol | Unused import IERC3156FlashLender | PENDING | |
| 81 | A06-P4-2 | P4 | GenericPoolOrderBookV6FlashBorrower.sol | Unused import IERC3156FlashBorrower | PENDING | |
| 82 | A07-P4-1 | P4 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Unused import Address | PENDING | |
| 83 | A07-P4-2 | P4 | RouteProcessorOrderBookV6ArbOrderTaker.sol | No explicit remapping for sushixswap-v2 | PENDING | |
| 84 | A15-P4-1 | P4 | Deploy.sol | Unused import OrderBookV6 | PENDING | |
| 85 | A15-P4-2 | P4 | Deploy.sol | Imports through concrete contract rather than canonical source | PENDING | |
| 86 | A15-P4-3 | P4 | Deploy.sol | Inconsistent address() cast on raindex variable | PENDING | |
| 87 | A15-P4-4 | P4 | Deploy.sol | Missing "memory-safe" annotation on second assembly block | PENDING | |
| 88 | A15-P4-5 | P4 | Deploy.sol | Repeated TaskV2/EvaluableV4 zero-value boilerplate | PENDING | |
| 89 | A01-P5-1 | P5 | OrderBookV6ArbCommon.sol | NatSpec @param tasks does not match field name task | PENDING | Dup of A01-P3-1 |
| 90 | A03-P5-1 | P5 | OrderBookV6FlashBorrower.sol | BadInitiator NatSpec inaccurately describes the check | PENDING | |
| 91 | A03-P5-2 | P5 | OrderBookV6FlashBorrower.sol | NatSpec for arb4 describes stale "access gate" evaluation | PENDING | Dup of A03-P3-2 |
| 92 | A04-P5-1 | P5 | OrderBookV6FlashLender.sol | maxFlashLoan NatSpec falsely claims active-debt disabling | PENDING | Dup of A04-P3-1 |
| 93 | A05-P5-01 | P5 | GenericPoolOrderBookV6ArbOrderTaker.sol | Misleading "Allow receiving gas" comment on non-payable fallback | PENDING | Dup of A05-3 |
| 94 | A06-P5-01 | P5 | GenericPoolOrderBookV6FlashBorrower.sol | Misleading "Allow receiving gas" comment on non-payable fallback | PENDING | Dup of A05-3 |
| 95 | A07-P5-01 | P5 | RouteProcessorOrderBookV6ArbOrderTaker.sol | Misleading "Allow receiving gas" comment on non-payable fallback | PENDING | Dup of A05-3 |
| 96 | A15-P5-3 | P5 | Deploy.sol | No require/revert on create returning address(0) (confirms A15-2) | PENDING | Dup of A15-2 |
| 97 | A15-P5-4 | P5 | Deploy.sol | No mechanism to reject unknown suite values | PENDING | |
