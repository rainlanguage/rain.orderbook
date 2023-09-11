# Orderbook svelte components

A minimal library of components for building Rain Orderbook applications using Svelte.

The goal of this library is to be totally headless, with no opinionated styling or markup whatsoever.

Currently contains:

- Common subgraph queries, with filters as Svelte stores
- Svelte stores that wrap wagmi methods for interacting with Orderbook and ERC20 tokens
- Utilities for boilerplate associated with building Orderbook GUIs

Use with [svelte-wagmi-stores](https://www.npmjs.com/package/svelte-wagmi-stores).

**This library is currently under active development and is subject to frequent breaking changes. It is recommended to pin dependencies to a specific version.**

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

Everything inside `src/lib` is part of the library, everything inside `src/routes` is for demos.
