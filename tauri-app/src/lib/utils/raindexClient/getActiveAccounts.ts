import { raindexClient, activeAccountsItems } from '$lib/stores/settings';
import type { AccountCfg } from '@rainlanguage/orderbook';
import { get } from 'svelte/store';

export function getAllNetworks(): Record<string, AccountCfg> {
  const $raindexClient = get(raindexClient);
  if (!$raindexClient) {
    throw new Error('Raindex client is not initialized');
  }

  const $activeAccountsItems = get(activeAccountsItems);
  const allAccounts = $raindexClient.getAllAccounts();
  if (allAccounts.error) {
    throw new Error('Could not fetch accounts from Raindex client');
  }

  if (Object.keys($activeAccountsItems).length === 0) {
    return {};
  }
  return Object.fromEntries(
    Object.entries(allAccounts.value).filter(([key]) => key in $activeAccountsItems),
  );
}
