import type { WagmiContext } from '$lib/types/wagmi';
import { getContext } from 'svelte';

export const WAGMI_CONTEXT_KEY = 'wagmi-stores';

export function getWagmiContext() {
  return getContext<WagmiContext>(WAGMI_CONTEXT_KEY);
}