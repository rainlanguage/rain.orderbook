import { cachedWritableStore, type AppStoresInterface, type ConfigSource } from '@rainlanguage/ui-components';
import { get } from 'svelte/store';

/**
 * A persistent store that holds the application configuration settings.
 *
 * This store is saved to local storage and persists between sessions.
 *
 * @default undefined - No configuration is set by default
 * @returns A writable store containing the application configuration
 */
export const settings = cachedWritableStore<ConfigSource | undefined>(
	'settings',
	undefined,
	(value) => JSON.stringify(value),
	(str) => {
		try {
			return JSON.parse(str) as ConfigSource;
		} catch {
			return undefined;
		}
	}
);

/**
 * A persistent store that controls whether vaults with zero balance should be hidden in the UI.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default true - Zero balance vaults are hidden by default
 * @returns A writable store containing a boolean value
 */
export const hideZeroBalanceVaults = cachedWritableStore<boolean>(
	'settings.hideZeroBalanceVaults',
	true, // default value is true
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : true;
		} catch {
			return true;
		}
	}
);

/**
 * A persistent store that controls whether to show only the user's items in lists.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default false - All items are shown by default
 * @returns A writable store containing a boolean value
 */
export const showMyItemsOnly = cachedWritableStore<boolean>(
	'settings.showMyItemsOnly',
	false,
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : false;
		} catch {
			return false;
		}
	}
);

/**
 * A persistent store that holds active subgraph URLs for different networks/orderbooks.
 *
 * This store maps network/orderbook identifiers to their corresponding subgraph URLs.
 * The setting is saved to local storage and persists between sessions.
 *
 * @default {} - Empty object by default
 * @returns A writable store containing a record of subgraph URLs
 */
export const activeSubgraphs = cachedWritableStore<Record<string, string>>(
	'settings.activeSubgraphs',
	{},
	JSON.stringify,
	(s) => {
		try {
			return JSON.parse(s);
		} catch {
			return {};
		}
	}
);

/**
 * A persistent store that holds the currently selected order hash.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default "" - Empty string by default
 * @returns A writable store containing the order hash string
 */
export const orderHash = cachedWritableStore<string>(
	'settings.orderHash',
	'',
	(value) => value,
	(str) => str || ''
);

/**
 * A persistent store that holds the currently show/hide setting for inactive orders.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default false - Inactive orders are hidden by default
 * @returns A writable store containing a boolean value
 */
export const activeOrderStatus = cachedWritableStore<boolean>(
	'settings.activeOrderStatus',
	true,
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : true;
		} catch {
			return true;
		}
	}
);

/**
 * Resets the active orderbook reference to the first available orderbook in the active network.
 * If no orderbooks are available, it sets the reference to undefined.
 * 
 * @param activeNetworkOrderbooks - A readable store containing the orderbooks for the active network
 * @param activeOrderbookRef - A writable store for the active orderbook reference
 * @returns A promise that resolves when the operation is complete
 */
export async function resetActiveOrderbookRef(activeNetworkOrderbooks: AppStoresInterface['activeNetworkOrderbooks'], activeOrderbookRef: AppStoresInterface['activeOrderbookRef']) {
  const $activeNetworkOrderbooks = get(activeNetworkOrderbooks);
  const $activeNetworkOrderbookRefs = Object.keys($activeNetworkOrderbooks);

  if ($activeNetworkOrderbookRefs.length > 0) {
    activeOrderbookRef.set($activeNetworkOrderbookRefs[0]);
  } else {
    activeOrderbookRef.set(undefined);
  }
}

/**
 * Resets the active network reference to the first available network in the settings.
 * If no networks are available, it sets the reference to undefined.
 * 
 * @param settings - A readable store containing the application settings
 * @param activeNetworkRef - A writable store for the active network reference
 * @returns A promise that resolves when the operation is complete
 */
export function resetActiveNetworkRef(settings: AppStoresInterface['settings'], activeNetworkRef: AppStoresInterface['activeNetworkRef']) {
  const $networks = get(settings)?.networks;

  if ($networks !== undefined && Object.keys($networks).length > 0) {
    activeNetworkRef.set(Object.keys($networks)[0]);
  } else {
    activeNetworkRef.set(undefined);
  }
}