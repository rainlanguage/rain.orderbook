# Rain Orderbook Development Instructions

Rain Orderbook (Raindex) is a complex multi-language project with Solidity smart contracts, Rust crates, TypeScript packages, and a Tauri desktop application. This project requires a Nix development environment for full functionality.

## Rules and environment assumptions

**NEVER CANCEL BUILDS OR LONG-RUNNING COMMANDS** - Many operations take 45+ minutes.

- **Nix is preinstalled**: always run commands via `nix develop -c <command>`.
- **Submodules are initialized**: the repo is checked out with `--recursive`.
- **Timeouts**: Use 60+ minutes for builds and 30+ minutes for tests.

## Development workflow (CI-prepared environment)

**ALWAYS follow these instructions first and fallback to additional search or bash commands only when you encounter unexpected information that does not match the info here.**

**All commands should be run within the Nix environment using `nix develop -c <command>` to match the CI workflows and ensure consistent dependency versions.**

The environment already has Nix installed and the repository is checked out with submodules. Focus on running the right builds/tests rather than installing tools.

Your goal is to pass all CI checks under `.github/workflows`. Therefore you must follow these instructions exactly, including: 
- Do not cancel long-running builds or tests. Full setup and validation can still take 45–90 minutes even with the prepared environment. Use timeouts of 60–90 minutes for builds and 30+ minutes for tests.
- Always run targeted tests and linters for the components you modify after each change.
- During development, run the tests for the part you are working on (see instructions below)
- Before pushing any changes, ALWAYS perform a fresh full setup and run all checks to mirror CI

### Development loop (every change)
- Run targeted tests and linters for the components you changed.
- Rebuild dependencies as needed:
  - Rust powering `@rainlanguage/orderbook` → `nix develop -c npm run build:orderbook`
  - `@rainlanguage/ui-components` → `nix develop -c npm run build -w @rainlanguage/ui-components`

### Before pushing (mirror CI)
```bash
./prep-all.sh
nix develop -c npm run lint-format-check:all
nix develop -c npm run build:orderbook   # if Rust/orderbook changed
nix develop -c npm run build:ui
nix develop -c cargo test --workspace
nix develop -c npm run test
nix develop -c forge test
```

## Architecture Notes

The project uses wasm-bindgen to create TypeScript bindings from Rust crates. This allows:
- Shared logic between desktop (Tauri) and web applications
- Publishing to npm for external developers
- Type-safe interaction with blockchain components

Therefore, you MUST respect the build dependency chain:
  - If you change Rust code used by `@rainlanguage/orderbook`, you must rebuild it before it can be used by `@rainlanguage/ui-components` or `@rainlanguage/webapp`:
    ```bash
    nix develop -c npm run build:orderbook
    ```
  - If you change `@rainlanguage/ui-components`, rebuild it before using it in `@rainlanguage/webapp`:
    ```bash
    nix develop -c npm run build -w @rainlanguage/ui-components
    ```

The complex build process reflects the multi-target nature (native, WASM, different platforms) and extensive submodule dependencies.

## Build System Overview

The project uses multiple build systems:
- **Nix** - Primary development environment and dependency management
- **Cargo** - Rust compilation for multiple targets including WASM
- **npm** - JavaScript/TypeScript package management and builds
- **Forge** - Solidity contract compilation

## Core build commands (reference)

### Individual Component Builds

#### Rust Components
```bash
# Build all Rust crates - takes 30+ minutes. NEVER CANCEL. Set timeout to 60+ minutes.
nix develop -c cargo build

# Test Rust code - takes 15+ minutes. NEVER CANCEL. Set timeout to 30+ minutes.
nix develop -c cargo test --workspace
```

#### TypeScript Packages
```bash
# Install npm dependencies - takes 2-5 minutes
nix develop -c npm install

# Build specific packages
nix develop -c npm run build:orderbook  # Builds @rainlanguage/orderbook package from the Rust code in the orderbook crate - takes 15-30 minutes
nix develop -c npm run build:ui        # Builds UI components and webapp - takes 5-10 minutes

# Build all packages - takes 20-45 minutes. NEVER CANCEL. Set timeout to 60+ minutes.
nix develop -c npm run build
```

#### Solidity Contracts
```bash
# Requires Nix environment for forge
nix develop -c forge build  # Takes 5-15 minutes
nix develop -c forge test   # Takes 10-20 minutes
```

## Running applications (reference)

### Web application
```bash
cd packages/webapp && nix develop -c npm run dev
# http://localhost:5173
```

### Tauri desktop application
```bash
nix develop .#tauri-shell --command cargo tauri dev
nix develop .#tauri-shell --command cargo tauri build
```

## Testing Commands

### Linting and Formatting
```bash
# Run all linting - takes 2-5 minutes
nix develop -c npm run lint:all

# Run all formatting - takes 1-2 minutes
nix develop -c npm run format:all

# Run all checks - takes 3-8 minutes
nix develop -c npm run check:all

# Combined lint, format, and check - takes 5-15 minutes
nix develop -c npm run lint-format-check:all
```

### Test Suites
```bash
# JavaScript/TypeScript tests
nix develop -c npm run test  # All packages - takes 10-20 minutes

# Individual package tests
nix develop -c npm run test -w @rainlanguage/webapp      # Takes 3-5 minutes
nix develop -c npm run test -w @rainlanguage/ui-components  # Takes 2-4 minutes

# Rust tests - REQUIRES network access for some dependencies
nix develop -c cargo test --workspace # Takes 5-10 minutes
```

## Project Structure

### Key Directories
- `src/` - Solidity smart contracts
- `crates/` - Rust libraries and tooling (11 crates total)
- `packages/orderbook/` - TypeScript bindings to Rust (published to npm)
- `packages/ui-components/` - Shared Svelte components
- `packages/webapp/` - SvelteKit web application
- `tauri-app/` - Cross-platform desktop application
- `lib/` - Git submodules (rain.interpreter, rain.metadata, etc.)

### Test Files
- **Rust tests:** 8 test files in `crates/*/tests/` and inline tests
- **TypeScript tests:** 1600+ test files across packages
- **Solidity tests:** In `test/` directory using Forge

## Build Time Summary

| Command | Expected Time | Timeout Setting |
|---------|--------------|-----------------|
| `./prep-all.sh` | 45-75 minutes | 90+ minutes |
| `nix develop -c cargo build` | 30-45 minutes | 60+ minutes |
| `nix develop -c npm run build` | 20-45 minutes | 60+ minutes |
| `nix develop -c cargo test --workspace` | 15-25 minutes | 30+ minutes |
| `nix develop -c npm run test` | 10-20 minutes | 30+ minutes |
| `nix develop -c npm run lint:all` | 2-5 minutes | 10+ minutes |

**CRITICAL REMINDER:** Never cancel builds or tests. Build failures are often due to network connectivity issues or missing environment setup, not code problems.