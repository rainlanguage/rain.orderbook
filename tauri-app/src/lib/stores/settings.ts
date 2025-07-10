import { derived, get } from '@square/svelte-store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import { parseYaml, type Address, type Hex, type NewConfig } from '@rainlanguage/orderbook';

export const EMPTY_SETTINGS: NewConfig = {
  orderbook: {
    version: '1',
    networks: {},
    subgraphs: {},
    metaboards: {},
    orderbooks: {},
    accounts: {},
    tokens: {},
    deployers: {},
  },
  dotrainOrder: {
    orders: {},
    scenarios: {},
    charts: {},
    deployments: {},
  },
};

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
export const settings = cachedWritableStore<NewConfig>(
  'settings',
  EMPTY_SETTINGS,
  (value) => JSON.stringify(value),
  () => {
    try {
      const text = get(settingsText);
      const res = parseYaml([text]);
      if (res.error) {
        throw new Error(res.error.readableMsg);
      }
      return res.value;
    } catch {
      return EMPTY_SETTINGS;
    }
  },
);
export const enableSentry = derived(settings, ($settings) =>
  $settings.orderbook.sentry !== undefined ? $settings.orderbook.sentry : true,
);

export const selectedChainIds = cachedWritableStore<number[]>(
  'settings.selectedChainIds',
  [],
  (value) => JSON.stringify(value),
  (str) => JSON.parse(str),
);

// accounts
export const accounts = derived(settings, ($settings) => $settings.orderbook.accounts ?? {});
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
export const activeAccounts = derived(
  [accounts, activeAccountsItems],
  ([$accounts, $activeAccountsItems]) =>
    Object.keys($activeAccountsItems).length === 0
      ? {}
      : Object.fromEntries(
          Object.entries($accounts).filter(([key]) => key in $activeAccountsItems),
        ),
);

// subgraphs
export const subgraphs = derived(settings, ($settings) =>
  $settings?.orderbook.subgraphs !== undefined ? Object.entries($settings.orderbook.subgraphs) : [],
);

// When networks / orderbooks settings updated, reset active network / orderbook
settings.subscribe(async () => {
  const $settings = get(settings);

  // Reset active account items if accounts have changed
  if (Object.keys($settings.orderbook.accounts ?? {}).length === 0) {
    activeAccountsItems.set({});
  } else {
    const currentActiveAccounts = get(activeAccountsItems);
    const updatedActiveAccounts = Object.fromEntries(
      Object.entries($settings.orderbook.accounts ?? {})
        .filter(([key, value]) => {
          if (key in currentActiveAccounts) {
            return currentActiveAccounts[key] === value.address;
          }
          return false;
        })
        .map(([key, value]) => [key, value.address as Address]),
    );
    activeAccountsItems.set(updatedActiveAccounts);
  }
});

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
