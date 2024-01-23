import  find from 'lodash/find';
import { derived, writable } from 'svelte/store';
import * as chains from 'viem/chains'

export const chainId = writable(parseInt(localStorage.getItem("settings.chainId") || '1'))

chainId.subscribe(value => {
  localStorage.setItem("settings.chainId", (value || 0).toString());
});

export const activeChain = derived(chainId, (val) => {  
  return find(Object.values(chains), (c) => c.id === val);
})