import type { ClientInit } from '@sveltejs/kit';
import type { HandleClientError } from '@sveltejs/kit';
import { init as initOrderbookPackage } from '@rainlanguage/orderbook/esm';

// in webapp, we need to initialize the wasm module in svelte init hook because
// safari doesn't support top-level await in multiple modules and sync wasm
// init would fail in other browsers (eg chrome) because of wasm size >8 MB.
// so instead of instantiating the wasm module at the top-level on orderbook pkg
// itself, we initialize it here in the webapp init hook, and for this we will
// have raindex webapp specific orderbook pkg build (see orderbook pkg build script),
// this can be reverted once safari supports top-level await in multiple modules
export const init: ClientInit = async () => {
    await initOrderbookPackage();
};

export const handleError: HandleClientError = ({ error, event }) => {
    console.error('Client hook error:', error, event);
};
