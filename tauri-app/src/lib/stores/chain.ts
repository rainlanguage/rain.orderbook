import  find from 'lodash/find';
import { derived, writable, get } from 'svelte/store';
import * as chains from 'viem/chains'
import { rpcUrl } from './settings';
import { toasts } from './toasts';
import { ToastMessageType } from '$lib/typeshare/toast';
import { invoke } from '@tauri-apps/api';

export const chainId = writable(parseInt(localStorage.getItem("settings.chainId") || '1'))

chainId.subscribe(value => {
  localStorage.setItem("settings.chainId", (value || 0).toString());
});

export async function updateChainId() {
  try {
    const val: number = await invoke('get_chainid', {rpcUrl: get(rpcUrl)});
    chainId.set(val);
  } catch(e) {
    toasts.add({
      message_type: ToastMessageType.Error,
      text: e as string
    });
  }
}

export const activeChain = derived(chainId, (val) => {  
  return find(Object.values(chains), (c) => c.id === val);
});

export const activeChainHasBlockExplorer = derived(activeChain, (val) => {
  return val && val.blockExplorers?.default !== undefined;
})

export function formatBlockExplorerTxUrl(txHash: string) {
  const c = get(activeChain);
  if(!c || !c.blockExplorers?.default) return;

  return `${c.blockExplorers?.default.url}/tx/${txHash}`;
}
