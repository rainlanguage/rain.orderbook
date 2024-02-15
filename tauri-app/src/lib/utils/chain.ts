import { get } from 'svelte/store';
import { rpcUrl, chainId } from '$lib/stores/settings';
import { invoke } from '@tauri-apps/api';

export async function setChainIdFromRpc() {
  if(!get(rpcUrl).isValid) return;

  const val: number = await invoke('get_chainid', {rpcUrl: get(rpcUrl).value});
  chainId.set(val);
}
