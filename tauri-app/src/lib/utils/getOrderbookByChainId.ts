import { settings } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function getOrderbookByChainId(chainId: number) {
  const appSettings = get(settings);
  const orderbook = Object.values(appSettings.orderbook.orderbooks).find(
    (n) => n.network.chainId === chainId,
  );
  if (!orderbook) {
    throw new Error('Network not found');
  }
  return orderbook;
}
