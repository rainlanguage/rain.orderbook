import { raindexClient } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function getAllNetworks() {
  const $raindexClient = get(raindexClient);
  if (!$raindexClient) {
    throw new Error('Raindex client is not initialized');
  }
  const networks = $raindexClient.getAllNetworks();
  if (networks.error) {
    throw new Error('Could not fetch networks from Raindex client');
  }
  return networks.value;
}
