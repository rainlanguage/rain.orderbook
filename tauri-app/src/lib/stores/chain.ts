import  find from 'lodash/find';
import { derived, writable } from 'svelte/store';
import { mainnet, polygon, flare } from 'viem/chains'

const SUPPORTED_CHAINS = [mainnet, polygon, flare];

export const chainId = writable(parseInt(localStorage.getItem("settings.chainId") || '1'))

chainId.subscribe(value => {
  localStorage.setItem("settings.chainId", (value || 0).toString());
});

export const isChainIdSupported = derived(chainId, (val) => SUPPORTED_CHAINS.map((c) => c.id).includes(val));

export const activeChain = derived(chainId, (val) => {  
  return find(SUPPORTED_CHAINS, (c) => c.id === val);
})