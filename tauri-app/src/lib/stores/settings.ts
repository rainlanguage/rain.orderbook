import { get, writable } from '@square/svelte-store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import { type Address, type Hex } from '@rainlanguage/orderbook';

// general
export const settingsText = cachedWritableStore<string>(
  'settingsText',
  '',
  (s) => s,
  (s) => s,
);
export const settingsFile = textFileStore(
  'Orderbook Settings Yaml',
  ['yml', 'yaml'],
  get(settingsText),
);
settingsText.subscribe((value) => {
  const currentFileText = get(settingsFile).text;
  if (value && currentFileText !== value) {
    settingsFile.set({
      text: value,
      path: undefined,
      isLoading: false,
      isSaving: false,
      isSavingAs: false,
      isEmpty: value.length === 0,
    });
  }
});

export const selectedChainIds = cachedWritableStore<number[]>(
  'settings.selectedChainIds',
  [],
  (value) => JSON.stringify(value),
  (str) => JSON.parse(str),
);

// accounts
export const activeAccountsItems = cachedWritableStore<Record<string, Address>>(
  'settings.activeAccountsItems',
  {},
  JSON.stringify,
  (s) => {
    try {
      return JSON.parse(s);
    } catch {
      return {};
    }
  },
);

export const showInactiveOrders = cachedWritableStore<boolean>(
  'settings.showInactiveOrders',
  true,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      const value = JSON.parse(str);
      return typeof value === 'boolean' ? value : true;
    } catch {
      return true;
    }
  },
);

export const hideZeroBalanceVaults = cachedWritableStore<boolean>(
  'settings.hideZeroBalanceVaults',
  true, // default value is true
  (value) => JSON.stringify(value),
  (str) => {
    try {
      return JSON.parse(str) as boolean;
    } catch {
      return true;
    }
  },
);

export const orderHash = cachedWritableStore<Hex>(
  'settings.orderHash',
  // @ts-expect-error initially the value is empty
  '',
  (value) => value,
  (str) => (str || '') as Hex,
);

/**
 * Store for managing selected token addresses for filtering
 * Stores an array of token addresses that are currently selected for filtering
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
  },
);

export const isSentryEnabled = writable<boolean>(false);
