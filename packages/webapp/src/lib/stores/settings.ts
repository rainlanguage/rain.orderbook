import { cachedWritableStore } from '@rainlanguage/ui-components';
import { type Address, type Hex, type NewConfig } from '@rainlanguage/orderbook';

export const EMPTY_CONFIG: NewConfig = {
	orderbook: {
		version: '1',
		subgraphs: {},
		networks: {},
		metaboards: {},
		orderbooks: {},
		tokens: {},
		deployers: {}
	},
	dotrainOrder: {
		deployments: {},
		orders: {},
		scenarios: {},
		charts: {}
	}
};

/**
 * A persistent store that holds the application configuration settings.
 *
 * This store is saved to local storage and persists between sessions.
 *
 * @default {} - No configuration is set by default
 * @returns A writable store containing the application configuration
 */
export const settings = cachedWritableStore<NewConfig>(
	'settings',
	EMPTY_CONFIG,
	(value) => JSON.stringify(value),
	(str) => {
		try {
			return JSON.parse(str) as NewConfig;
		} catch {
			return EMPTY_CONFIG;
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
 * A persistent store that holds the currently selected chain IDs.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default [] - Empty array by default
 * @returns A writable store containing an array of chain IDs
 */
export const selectedChainIds = cachedWritableStore<number[]>(
	'settings.selectedChainIds',
	[],
	(value) => JSON.stringify(value),
	(str) => JSON.parse(str)
);

/**
 * A persistent store that holds the currently selected order hash.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default "" - Empty string by default
 * @returns A writable store containing the order hash string
 */
export const orderHash = cachedWritableStore<Hex>(
	'settings.orderHash',
	// @ts-expect-error initially the value is empty
	'',
	(value) => value,
	(str) => (str || '') as Hex
);

/**
 * A persistent store that holds the currently show/hide setting for inactive orders.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default false - Inactive orders are hidden by default
 * @returns A writable store containing a boolean value
 */
export const showInactiveOrders = cachedWritableStore<boolean>(
	'settings.showInactiveOrders',
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
 * A persistent store that holds the currently selected tokens for filtering.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default [] - Empty array by default
 * @returns A writable store containing selected tokens mapped by address
 */
export const activeTokens = cachedWritableStore<Address[]>(
	'settings.selectedTokens',
	[],
	JSON.stringify,
	(str) => {
		try {
			return JSON.parse(str);
		} catch {
			return [];
		}
	}
);
