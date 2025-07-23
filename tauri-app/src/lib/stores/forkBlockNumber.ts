import { getBlockNumberFromRpc } from '$lib/services/chain';
import { fetchableIntStore } from '$lib/storesGeneric/fetchableStore';
import { walletConnectNetwork } from '$lib/stores/walletconnect';
import { get } from 'svelte/store';
import { getNetworkByChainId } from '$lib/utils/raindexClient/getNetworkByChainId';

export const forkBlockNumber = fetchableIntStore('forkBlockNumber', async () => {
  const network = getNetworkByChainId(get(walletConnectNetwork));
  return getBlockNumberFromRpc(network.rpcs);
});
