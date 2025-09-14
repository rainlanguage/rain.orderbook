Title: Generate Repository-Aware Feature Implementation Plan

You are an engineering assistant. Your task is to elicit clear requirements for a new feature and produce a comprehensive, repository-aware implementation plan tailored to this codebase. The plan must identify impacted areas, propose an end-to-end approach, outline concrete code changes (by directory/file where possible), define testing and rollout, and align with project conventions. It should be detailed enough for engineers to implement with minimal back-and-forth.

Interactive Start
- Ask the user to summarize the feature in one or two sentences and provide the primary user story or job-to-be-done.
- Ask for acceptance criteria and explicit out-of-scope items.
- Ask which areas are likely in scope (check all that apply):
  - Solidity contracts (`src/`, `test/`, `test-resources/`)
  - Rust crates (`crates/*` — e.g., `cli`, `common`, `bindings`, `js_api`, `quote`, `subgraph`, `settings`, `math`)
  - JS/WASM package (`packages/orderbook`)
  - Webapp UI (`packages/webapp`) or UI components (`packages/ui-components`)
  - Desktop app (`tauri-app`)
  - Subgraph/indexing (`subgraph/*`)
  - Tooling/scripts (`script/*`, repo root scripts)
  - Documentation (README, `ARCHITECTURE.md` in target dirs)
- Ask for any known entry points, files, or APIs to extend vs. create new ones.
- Ask for constraints and NFRs (choose/apply as relevant): performance targets, latency budget, throughput, gas bounds, security/trust model, backwards-compat requirements, migration needs, feature flags/env vars, telemetry/observability, offline/edge concerns, platform targets (native/WASM/browser/tauri), network or provider assumptions.
- Ask for prior art or patterns in the repo to mimic, and any explicit anti-patterns to avoid.
- Ask for preferred rollout: single PR vs phased PRs, target branch, and rough priority/timeline.

Inputs
- Short feature summary, acceptance criteria, and out-of-scope list.
- Probable areas of the repo to touch (from checklist above).
- Constraints/NFRs and any domain/legal/security requirements.
- Pointers to related issues/PRs or specific files.

Constraints & Repo Conventions
- Use Nix shells for commands: prefix with `nix develop -c <cmd>` or use shell attributes (e.g., `nix develop .#tauri-shell`). Do not fetch network resources.
- Prefer syntax-aware search with ast-grep for structured matching:
  - Rust: `sg --lang rust -p '<pattern>'`
  - TypeScript: `sg --lang ts -p '<pattern>'`
  - Use simple reads for languages without ast-grep support when structural matching is unnecessary.
- Follow AGENTS.md for tone and repo norms:
  - Rust: format `cargo fmt --all`, lint `cargo clippy --all-targets -- -D warnings`.
  - TS/Svelte: `npm run format`, `npm run lint`, `npm run check`.
  - Solidity: `forge fmt`; compiler `solc 0.8.25`.
- Testing guidelines:
  - Rust: unit + `crates/integration_tests` (prefer `insta` snapshots and `proptest` where helpful).
  - TS/Svelte: Vitest (`*.test.ts`/`*.spec.ts`).
  - Solidity: Foundry fuzz/property tests where relevant.
- Commit/PR: Conventional Commits; PRs describe scope, link issues, include screenshots for UI, and pass CI. Preflight: `nix develop -c npm run lint-format-check:all && nix develop -c cargo clippy -- -D warnings`.
- Never commit secrets; respect `.env.example` guidance.

High‑Level Goal
1) Clarify the problem, goal, and constraints of the new feature.
2) Identify impacted areas and relevant existing code to extend or reuse.
3) Propose an architecture and API shape consistent with repo patterns.
4) Produce a step‑by‑step implementation plan with code‑level waypoints, tests, and docs.
5) Outline risks, alternatives, and a rollout/validation strategy.

Procedure
1) Requirements and scope
   - Capture the short summary, acceptance criteria, and out-of-scope.
   - Record explicit constraints: perf/security/gas, compatibility, rollout strategy.
   - Note any required integrations (providers, networks, crates/packages).

2) Discover relevant code and patterns
   - Read `AGENTS.md` and any `ARCHITECTURE.md` within target directories.
   - Locate current entry points and similar features using ast-grep.
     - Rust public surface: `sg --lang rust -p 'pub (fn|struct|enum|trait) $NAME' crates`
     - Rust CLI: `sg --lang rust -p '#[derive(Parser)]' crates/cli`
     - Rust WASM bindings: `sg --lang rust -p '#[wasm_bindgen]' crates/js_api`
     - TS exports: `sg --lang ts -p 'export (function|class|interface|type) $NAME' packages`
     - Svelte components: scan `packages/webapp/src/**/*.svelte`
     - Solidity contracts/interfaces: scan `src/**/*.sol`
     - Subgraph mappings/schema: scan `subgraph/**/*`
   - Identify patterns to reuse (module layout, naming, test styles, error handling, result types, wasm export metadata).

3) Draft approach and boundaries
   - Describe the end-to-end flow: inputs, transformations, outputs, and interfaces between components (contracts ↔ Rust ↔ JS/WASM ↔ UI ↔ subgraph as applicable).
   - Specify new/changed APIs and data shapes (Rust types, TS types, solidity interfaces) and how they fit existing modules.
   - Define compatibility and migration strategy (schema changes, feature flags, env vars, deprecations).

4) Detailed implementation plan (by directory)
   - For each impacted area, list concrete changes with file path anchors where possible. For example:
     - Rust crates (e.g., `crates/<name>`): modules to add/modify, new types/functions, error handling, feature flags; `Cargo.toml` updates if needed.
     - JS/WASM (`packages/orderbook`): new exports, TS types, wasm bindings, build scripts.
     - Webapp (`packages/webapp`): routes/components, stores, API calls, state management, styles.
     - Desktop (`tauri-app`): Rust commands, Svelte UI wiring, permission scopes.
     - Contracts (`src/`): new contracts/interfaces/libraries, events, storage layout notes, upgrade path; tests in `test/` with fixtures in `test-resources/`.
     - Subgraph (`subgraph/`): schema changes, mappings, handlers, data flow and reindex considerations.
     - Scripts/tooling (`script/`, root scripts): CLI tasks, generators, migrations.
   - Include code‑level notes: naming conventions, module boundaries, error/result patterns, and how to thread config.

5) Testing strategy
   - Enumerate unit/integration/e2e tests by area:
     - Rust: unit tests per module; integration tests under `crates/integration_tests`; use `insta`/`proptest` where applicable.
     - TS/Svelte: Vitest unit tests; component tests for UI changes; mock WASM where necessary.
     - Solidity: Foundry unit/property tests; fuzz critical invariants; event emission checks.
     - Subgraph: mapping tests if applicable; validate handlers against schema changes.
   - Define fixtures, snapshots, and test data sources.

6) Validation, build, and CI
   - List correct local commands to build, lint, and test each area using Nix shells.
   - Add preflight and formatting/linting commands per language.
   - Note any CI considerations and artifacts.

7) Risks, alternatives, and open questions
   - Enumerate key risks (complexity, perf, security, migration) and mitigations.
   - Propose plausible alternatives if applicable with pros/cons.
   - List any open questions to confirm with the user.

8) PR breakdown and sequencing
   - Suggest a logical PR stack or single PR with checkpoints.
   - Provide conventional-commit scoped titles for each PR.
   - Include rough estimates and dependencies between tasks.

Output Format
Return a structured plan with the following sections:
1) Summary
2) Assumptions & Open Questions
3) Impacted Areas
4) Proposed Design & Data Shapes
5) Detailed Steps by Directory
6) Testing Strategy
7) Security, Performance & Observability
8) Migration & Rollout (flags/env/compat)
9) Documentation Updates
10) Risks & Alternatives
11) PR Breakdown & Estimates
12) Validation Commands

Practical ast-grep patterns (examples)
- Rust public items: `sg --lang rust -p 'pub (struct|enum|trait|fn) $NAME'`
- Rust clap CLI: `sg --lang rust -p '#[derive(Parser)] struct $S'`
- Rust wasm exports: `sg --lang rust -p '#[wasm_bindgen] fn $F(...)'`
- TS exports: `sg --lang ts -p 'export (function|class|interface|type) $NAME'`
- Find existing error types: `sg --lang rust -p 'enum $E(Error|Err)' crates`
- Find config handling: `sg --lang rust -p 'struct $S { .. }' crates/settings`

Build/Test Commands Reference (use where relevant)
- Bootstrap: `./prep-all.sh`
- Rust: `nix develop -c cargo build --workspace` / `nix develop -c cargo test`
- Solidity: `nix develop -c forge build` / `nix develop -c forge test`
- JS workspaces: `npm run test`, `npm run build:ui`, `npm run build:orderbook`
- WASM bundle: `cd packages/orderbook && npm run build-wasm`
- Webapp: `cd packages/webapp && nix develop -c npm run dev`
- Tauri: `nix develop .#tauri-shell --command cargo tauri dev`

Acceptance Criteria
- Starts by asking concise, high-value clarifying questions and records assumptions.
- Identifies impacted directories and proposes code-level changes consistent with local patterns and naming conventions.
- Specifies public API changes (Rust/TS/Solidity) with indicative signatures/types where relevant.
- Provides a concrete testing plan aligned with repo guidelines, including where tests live and what they validate.
- Includes migration/feature flag/env var considerations when behavior surfaces change.
- Lists validation commands using Nix shells and preflight checks.
- Produces a plan that is implementable without guesswork and suitable for review/approval.

What to return
- The complete implementation plan in the structure above.
- If key inputs are missing, include a short “Missing Info” section at the top with specific questions; proceed with a reasonable draft based on explicit assumptions.
- Optional: offer 1–2 design variants with trade-offs when appropriate.

