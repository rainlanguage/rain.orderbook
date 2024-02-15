import { get } from 'svelte/store';
import { forkBlockNumber, rpcUrl } from '$lib/stores/settings';
import { invoke } from '@tauri-apps/api';

export async function setForkBlockNumberFromRpc() {
  if(!get(rpcUrl).isValid) return;

  try {
    const val: number = await invoke('get_block_number', {rpcUrl: get(rpcUrl).value});
    forkBlockNumber.set(val);
  // eslint-disable-next-line no-empty
  } catch(e) {}
}
