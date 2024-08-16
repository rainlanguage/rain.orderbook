import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, subgraphUrl } from '$lib/stores/settings';

export async function batchOrderQuotes(
  orderHashes: string[],
): Promise<{ maxOutput: string; ratio: string }[]> {
  return invoke('batch_order_quotes', {
    orderHashes,
    subgraphUrl: get(subgraphUrl),
    rpcUrl: get(rpcUrl),
    orderbook: get(orderbookAddress),
  });
}
