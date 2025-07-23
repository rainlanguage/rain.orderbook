import { raindexClient } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function getNetworkByChainId(chainId: number) {
  const $raindexClient = get(raindexClient);
  if (!$raindexClient) {
    throw new Error('Raindex client is not initialized');
  }
  const network = $raindexClient.getNetworkByChainId(chainId);
  if (network.error) {
    throw new Error('Could not fetch network from Raindex client');
  }
  return network.value;
}
