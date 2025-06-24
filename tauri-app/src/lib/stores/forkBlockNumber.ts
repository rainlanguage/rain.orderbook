import { getBlockNumberFromRpc } from '$lib/services/chain';
import { fetchableIntStore } from '$lib/storesGeneric/fetchableStore';
import { get } from 'svelte/store';
import { rpcUrls } from './settings';

export const forkBlockNumber = fetchableIntStore('forkBlockNumber', async () => {
  const $rpcUrls = get(rpcUrls);
  if (!$rpcUrls) return 0;

  return getBlockNumberFromRpc($rpcUrls);
});

// When active chain updated, reset active orderbook
rpcUrls.subscribe(async () => {
  await rpcUrls.load();
  forkBlockNumber.fetch();
});
