import { raindexClient } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function isSentryEnabled() {
  const $raindexClient = get(raindexClient);
  if (!$raindexClient) {
    throw new Error('Raindex client is not initialized');
  }
  const flag = $raindexClient.isSentryEnabled();
  if (flag.error) {
    throw new Error('Could not fetch sentry field from Raindex client');
  }
  return flag.value;
}
