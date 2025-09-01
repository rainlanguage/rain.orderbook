# Rain Orderbook Development Instructions

Rain Orderbook (Raindex) is a complex multi-language project with Solidity smart contracts, Rust crates, TypeScript packages, and a Tauri desktop application. This project requires a Nix development environment for full functionality.

**ALWAYS follow these instructions first and fallback to additional search or bash commands only when you encounter unexpected information that does not match the info here.**

## Critical Prerequisites

**NEVER CANCEL BUILDS OR LONG-RUNNING COMMANDS** - Many operations take 45+ minutes. Always set timeouts of 60+ minutes for builds and 30+ minutes for tests.

### Required Tools and Environment

- **Nix package manager** - REQUIRED for full development workflow
- **Node.js 22+** - The project requires Node.js 22+ (packages/orderbook specifies "engines": {"node": ">=22"})
- **Rust with WASM target** - Required for building the TypeScript bindings
- **Git with submodules** - Critical: project has extensive git submodules that must be initialized

### Environment Setup Commands

```bash
# 1. Initialize git submodules (CRITICAL - required for any build to work)
git submodule update --init --recursive

# 2. Set up environment files
cp .env.example .env
cp tauri-app/.env.example tauri-app/.env
cp packages/webapp/.env.example packages/webapp/.env
# Fill out VITE_WALLETCONNECT_PROJECT_ID and PUBLIC_WALLETCONNECT_PROJECT_ID with test project IDs from Reown

# 3. Install Rust WASM target (required for orderbook package build)
rustup target add wasm32-unknown-unknown

# 4. Make setup script executable and run full setup
chmod +x prep-all.sh
```

## Build System Overview

The project uses multiple build systems:
- **Nix** - Primary development environment and dependency management
- **Cargo** - Rust compilation for multiple targets including WASM
- **npm** - JavaScript/TypeScript package management and builds
- **Forge** - Solidity contract compilation

## Core Build Commands

### Full Project Setup (RECOMMENDED)
```bash
# Complete project setup - takes 45+ minutes. NEVER CANCEL. Set timeout to 90+ minutes.
./prep-all.sh
```

**Build time expectation:** 45-75 minutes. This script:
- Installs Forge dependencies
- Sets up all submodules (rain.interpreter, rain.metadata, etc.)
- Builds Rust crates for multiple targets
- Builds all npm packages
- Sets up Tauri environment

### Individual Component Builds

#### Rust Components
```bash
# Build all Rust crates - takes 30+ minutes. NEVER CANCEL. Set timeout to 60+ minutes.
cargo build

# Build for WASM target (required for orderbook package) - takes 20+ minutes
cargo build --target wasm32-unknown-unknown --lib -r --workspace --exclude rain_orderbook_cli --exclude rain_orderbook_integration_tests

# Test Rust code - takes 15+ minutes. NEVER CANCEL. Set timeout to 30+ minutes.
cargo test --workspace
```

#### TypeScript Packages
```bash
# Install npm dependencies - takes 2-5 minutes
npm install

# Build specific packages
npm run build:orderbook  # Builds @rainlanguage/orderbook package - takes 15-30 minutes
npm run build:ui        # Builds UI components and webapp - takes 5-10 minutes

# Build all packages - takes 20-45 minutes. NEVER CANCEL. Set timeout to 60+ minutes.
npm run build
```

#### Solidity Contracts
```bash
# Requires Nix environment for forge
nix develop -c forge build  # Takes 5-15 minutes
nix develop -c forge test   # Takes 10-20 minutes
```

## Running Applications

### Web Application
```bash
# Start development server
cd packages/webapp && nix develop -c npm run dev
# Accessible at http://localhost:5173
```

### Tauri Desktop Application
```bash
# Development mode
nix develop .#tauri-shell --command cargo tauri dev

# Build production version - takes 20-30 minutes
nix develop .#tauri-shell --command cargo tauri build
```

## Testing Commands

### Linting and Formatting
```bash
# Run all linting (works without Nix) - takes 2-5 minutes
npm run lint:all

# Run all formatting (works without Nix) - takes 1-2 minutes
npm run format:all

# Run all checks - takes 3-8 minutes
npm run check:all

# Combined lint, format, and check - takes 5-15 minutes
npm run lint-format-check:all
```

### Test Suites
```bash
# JavaScript/TypeScript tests
npm run test  # All packages - takes 10-20 minutes

# Individual package tests
npm run test -w @rainlanguage/webapp      # Takes 3-5 minutes
npm run test -w @rainlanguage/ui-components  # Takes 2-4 minutes

# Rust tests - REQUIRES network access for some dependencies
cargo test --package rain_orderbook_common  # Takes 5-10 minutes
```

## Validation Requirements

### Before Committing Changes
ALWAYS run these validation steps in order:

1. **Lint and format:** `npm run lint-format-check:all` (5-15 minutes)
2. **Build core packages:** `npm run build:orderbook` (15-30 minutes) 
3. **Test changed components:** Run relevant test suites
4. **Manual validation:** If changing UI, start webapp/tauri and test user flows

### Complete Build Validation
For major changes, run full validation:

```bash
# Full project rebuild - 60-90 minutes total. NEVER CANCEL.
./prep-all.sh

# Verify all tests pass - 20-30 minutes
npm run test
nix develop -c cargo test --workspace
```

## Common Issues and Solutions

### Network Dependencies
- Some Rust builds fail due to network requirements (Solidity compiler downloads)
- If build fails with DNS errors, this is expected in restricted network environments
- Document any consistently failing commands with specific error messages

### Node.js Version
- Project requires Node.js 22+
- If using older Node.js, expect engine warnings and potential build failures
- Use `node --version` to verify

### Git Submodules
- **CRITICAL:** Always run `git submodule update --init --recursive` after fresh clone
- Missing submodules cause immediate build failures with "No such file or directory" errors

### WASM Target
- TypeScript bindings require Rust WASM target: `rustup target add wasm32-unknown-unknown`
- Without this, orderbook package build fails immediately

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
| `cargo build` | 30-45 minutes | 60+ minutes |
| `npm run build` | 20-45 minutes | 60+ minutes |
| `cargo test --workspace` | 15-25 minutes | 30+ minutes |
| `npm run test` | 10-20 minutes | 30+ minutes |
| `npm run lint:all` | 2-5 minutes | 10+ minutes |

**CRITICAL REMINDER:** Never cancel builds or tests. Build failures are often due to network connectivity issues or missing environment setup, not code problems.

## Architecture Notes

The project uses wasm-bindgen to create TypeScript bindings from Rust crates. This allows:
- Shared logic between desktop (Tauri) and web applications
- Publishing to npm for external developers
- Type-safe interaction with blockchain components

The complex build process reflects the multi-target nature (native, WASM, different platforms) and extensive submodule dependencies.