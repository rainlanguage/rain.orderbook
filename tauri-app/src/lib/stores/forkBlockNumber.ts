import { getBlockNumberFromRpc } from '$lib/services/chain';
import { fetchableIntStore } from '$lib/storesGeneric/fetchableStore';
import { getOrderbookByChainId } from '$lib/utils/getOrderbookByChainId';
import { walletConnectNetwork } from '$lib/stores/walletconnect';
import { get } from 'svelte/store';

export const forkBlockNumber = fetchableIntStore('forkBlockNumber', async () => {
  const orderbook = getOrderbookByChainId(get(walletConnectNetwork));
  return getBlockNumberFromRpc(orderbook.network.rpcs);
});
