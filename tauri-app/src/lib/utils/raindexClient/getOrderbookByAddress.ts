import { raindexClient } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function getOrderbookByAddress(address?: string) {
  if (!address) {
    throw new Error('Address is required to get orderbook');
  }

  const $raindexClient = get(raindexClient);
  if (!$raindexClient) {
    throw new Error('Raindex client is not initialized');
  }

  const orderbook = $raindexClient.getOrderbookByAddress(address);
  if (orderbook.error) {
    throw new Error(`Orderbook not found for address: ${address}`);
  }

  return orderbook.value;
}
