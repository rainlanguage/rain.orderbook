import type { WagmiContext } from '$lib/types/wagmi';
import { getContext, setContext } from 'svelte';

export const WAGMI_CONTEXT_KEY = 'wagmi-stores';

export function setWagmiContext(stores: WagmiContext) {
  setContext(WAGMI_CONTEXT_KEY, stores);
}

export function getWagmiContext() {
  return getContext<WagmiContext>(WAGMI_CONTEXT_KEY);
}