import { asyncDerived, derived, get } from '@square/svelte-store';
import { cachedWritableStore, cachedWritableStringOptional } from '@rainlanguage/ui-components';
import find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import {
  type ConfigSource,
  type OrderbookCfgRef,
  type OrderbookConfigSource,
} from '@rainlanguage/orderbook';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { pickBy } from 'lodash';
import { mockConfigSource } from '$lib/mocks/mockConfigSource';
import { beforeEach, describe } from 'vitest';

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
  },
);
export const enableSentry = derived(settings, ($settings) =>
  $settings?.sentry !== undefined ? $settings.sentry : true,
);

// networks
export const activeNetworkRef = cachedWritableStringOptional('settings.activeNetworkRef');
export const activeNetwork = asyncDerived(
  [settings, activeNetworkRef],
  async ([$settings, $activeNetworkRef]) => {
    return $activeNetworkRef !== undefined && $settings?.networks !== undefined
      ? $settings.networks[$activeNetworkRef]
      : undefined;
  },
);
export const rpcUrl = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.rpc);
export const chainId = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.['chain-id']);
export const activeChain = derived(chainId, ($activeChainId) =>
  find(Object.values(chains), (c) => c.id === $activeChainId),
);
export const activeChainHasBlockExplorer = derived(activeChain, ($activeChain) => {
  return $activeChain && $activeChain?.blockExplorers?.default !== undefined;
});
export const activeChainLatestBlockNumber = derived(activeNetwork, ($activeNetwork) =>
  $activeNetwork !== undefined ? getBlockNumberFromRpc($activeNetwork.rpc) : 0,
);

// orderbook
export const activeOrderbookRef = cachedWritableStringOptional('settings.activeOrderbookRef');
export const activeNetworkOrderbooks = derived(
  [settings, activeNetworkRef],
  ([$settings, $activeNetworkRef]) =>
    $settings?.orderbooks
      ? (pickBy(
          $settings.orderbooks,
          (orderbook) => orderbook.network === $activeNetworkRef,
        ) as Record<OrderbookCfgRef, OrderbookConfigSource>)
      : ({} as Record<OrderbookCfgRef, OrderbookConfigSource>),
);
export const activeOrderbook = derived(
  [settings, activeOrderbookRef],
  ([$settings, $activeOrderbookRef]) =>
    $settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined
      ? $settings.orderbooks[$activeOrderbookRef]
      : undefined,
);
export const subgraphUrl = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
  $settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
    ? $settings.subgraphs[$activeOrderbook.subgraph]
    : undefined,
);
export const orderbookAddress = derived(
  activeOrderbook,
  ($activeOrderbook) => $activeOrderbook?.address,
);

export const hasRequiredSettings = derived(
  [activeNetworkRef, activeOrderbookRef],
  ([$activeNetworkRef, $activeOrderbookRef]) =>
    $activeNetworkRef !== undefined && $activeOrderbookRef !== undefined,
);

// accounts
export const accounts = derived(settings, ($settings) => $settings?.accounts ?? {});
export const activeAccountsItems = cachedWritableStore<Record<string, string>>(
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
export const subgraph = derived(settings, ($settings) =>
  $settings?.subgraphs !== undefined ? Object.entries($settings.subgraphs) : [],
);
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
  },
);

// When networks / orderbooks settings updated, reset active network / orderbook
settings.subscribe(async () => {
  const $settings = get(settings);
  const $activeNetworkRef = get(activeNetworkRef);
  const $activeOrderbookRef = get(activeOrderbookRef);

  if (
    $settings?.networks === undefined ||
    $activeNetworkRef === undefined ||
    ($settings?.networks !== undefined &&
      $activeNetworkRef !== undefined &&
      !Object.keys($settings.networks).includes($activeNetworkRef))
  ) {
    resetActiveNetworkRef();
  }

  if (
    !$settings?.orderbooks === undefined ||
    $activeOrderbookRef === undefined ||
    ($settings?.orderbooks !== undefined &&
      $activeOrderbookRef !== undefined &&
      !Object.keys($settings.orderbooks).includes($activeOrderbookRef))
  ) {
    resetActiveOrderbookRef();
  }

  // Reset active account items if accounts have changed
  if ($settings?.accounts === undefined) {
    activeAccountsItems.set({});
  } else {
    const currentActiveAccounts = get(activeAccountsItems);
    const updatedActiveAccounts = Object.fromEntries(
      Object.entries($settings.accounts ?? {}).filter(([key, value]) => {
        if (key in currentActiveAccounts) {
          return currentActiveAccounts[key] === value;
        }
        return false;
      }),
    );
    activeAccountsItems.set(updatedActiveAccounts);
  }

  // Reset active subgraphs if subgraphs have changed
  if ($settings?.subgraphs === undefined) {
    activeSubgraphs.set({});
  } else {
    const currentActiveSubgraphs = get(activeSubgraphs);
    const updatedActiveSubgraphs = Object.fromEntries(
      Object.entries($settings.subgraphs).filter(([key, value]) => {
        if (key in currentActiveSubgraphs) {
          return currentActiveSubgraphs[key] === value;
        }
        return false;
      }),
    );
    activeSubgraphs.set(updatedActiveSubgraphs);
  }
});

// When active network is updated to undefined, reset active orderbook
activeNetworkRef.subscribe(($activeNetworkRef) => {
  if ($activeNetworkRef === undefined) {
    resetActiveOrderbookRef();
  }
});

// When active network is updated to not include active orderbook, reset active orderbook
activeNetworkOrderbooks.subscribe(async ($activeNetworkOrderbooks) => {
  const $activeOrderbookRef = get(activeOrderbookRef);

  if (
    $activeOrderbookRef !== undefined &&
    !Object.keys($activeNetworkOrderbooks).includes($activeOrderbookRef)
  ) {
    resetActiveOrderbookRef();
  }
});

// reset active orderbook to first available, otherwise undefined
export function resetActiveOrderbookRef() {
  const $activeNetworkOrderbooks = get(activeNetworkOrderbooks);
  const $activeNetworkOrderbookRefs = Object.keys($activeNetworkOrderbooks);

  if ($activeNetworkOrderbookRefs.length > 0) {
    activeOrderbookRef.set($activeNetworkOrderbookRefs[0]);
  } else {
    activeOrderbookRef.set(undefined);
  }
}

// reset active orderbook to first available, otherwise undefined
export async function resetActiveNetworkRef() {
  const $networks = get(settings)?.networks;

  if ($networks !== undefined && Object.keys($networks).length > 0) {
    activeNetworkRef.set(Object.keys($networks)[0]);
  } else {
    activeNetworkRef.set(undefined);
  }
}

export const activeOrderStatus = cachedWritableStore<boolean | undefined>(
  'settings.activeOrderStatus',
  undefined,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      const parsed = JSON.parse(str);
      return typeof parsed === 'boolean' ? parsed : undefined;
    } catch {
      return undefined;
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

export const orderHash = cachedWritableStore<string>(
  'settings.orderHash',
  '',
  (value) => value,
  (str) => str || '',
);
if (import.meta.vitest) {
  const { test, expect } = import.meta.vitest;

  describe('Settings active accounts items', () => {
    // Reset store values before each test to prevent state leakage
    beforeEach(() => {
      settings.set(undefined);
      activeAccountsItems.set({});
      activeSubgraphs.set({});
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set(undefined);

      settings.set(mockConfigSource);
      activeAccountsItems.set({
        name_one: 'address_one',
        name_two: 'address_two',
      });
      activeSubgraphs.set({
        mainnet: 'https://api.thegraph.com/subgraphs/name/mainnet',
      });

      // Verify initial state
      expect(get(settings)).toEqual(mockConfigSource);
      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
        name_two: 'address_two',
      });
      expect(get(activeSubgraphs)).toEqual({
        mainnet: 'https://api.thegraph.com/subgraphs/name/mainnet',
      });
    });

    test('should remove account if that account is removed', () => {
      // Test removing an account
      const newSettings = {
        ...mockConfigSource,
        accounts: {
          name_one: 'address_one',
        },
      };

      // Update settings - this should trigger the subscription
      settings.set(newSettings);

      // Check the expected result
      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
      });
    });

    test('should remove account if the value is different', () => {
      const newSettings = {
        ...mockConfigSource,
        accounts: {
          name_one: 'address_one',
          name_two: 'new_value',
        },
      };

      settings.set(newSettings);

      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
      });
    });

    test('should reset active accounts when accounts are undefined', () => {
      const newSettings = {
        ...mockConfigSource,
        accounts: undefined,
      };

      settings.set(newSettings);

      expect(get(activeAccountsItems)).toEqual({});
    });

    test('should update active subgraphs when subgraph value changes', () => {
      const newSettings = {
        ...mockConfigSource,
        subgraphs: {
          mainnet: 'new value',
        },
      };

      settings.set(newSettings);

      expect(get(activeSubgraphs)).toEqual({});
    });

    test('should update active subgraphs when subgraph removed', () => {
      const newSettings = {
        ...mockConfigSource,
        subgraphs: {
          testnet: 'testnet',
        },
      };

      settings.set(newSettings);

      expect(get(activeSubgraphs)).toEqual({});
    });

    test('should reset active subgraphs when subgraphs are undefined', () => {
      const newSettings = {
        ...mockConfigSource,
        subgraphs: undefined,
      };

      settings.set(newSettings);

      expect(get(activeSubgraphs)).toEqual({});
    });
  });

  describe('Network and Orderbook Management', () => {
    beforeEach(() => {
      // Reset all stores
      settings.set(undefined);
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set(undefined);
      activeAccountsItems.set({});
      activeSubgraphs.set({});
    });

    test('should reset activeNetworkRef when networks are undefined', () => {
      // First set valid settings
      settings.set(mockConfigSource);
      activeNetworkRef.set('mainnet');

      // Then make networks undefined
      const newSettings = {
        ...mockConfigSource,
        networks: undefined,
      };

      settings.set(newSettings);

      expect(get(activeNetworkRef)).toBeUndefined();
    });

    test('should reset activeOrderbookRef when activeNetworkRef is undefined', () => {
      settings.set(mockConfigSource);
      activeNetworkRef.set('mainnet');
      activeOrderbookRef.set('orderbook1');

      activeNetworkRef.set(undefined);

      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('resetActiveNetworkRef should set first available network', async () => {
      settings.set(mockConfigSource);

      await resetActiveNetworkRef();

      expect(get(activeNetworkRef)).toBe('mainnet');
    });

    test('resetActiveNetworkRef should set undefined when no networks', async () => {
      const emptySettings = { ...mockConfigSource, networks: {} };
      settings.set(emptySettings);

      await resetActiveNetworkRef();

      expect(get(activeNetworkRef)).toBeUndefined();
    });

    test('should reset activeOrderbookRef when orderbooks are undefined', () => {
      settings.set(mockConfigSource);
      activeOrderbookRef.set('orderbook1');

      const newSettings = {
        ...mockConfigSource,
        orderbooks: undefined,
      };

      settings.set(newSettings);

      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('should filter orderbooks by active network', () => {
      const multiNetworkConfig = {
        ...mockConfigSource,
        orderbooks: {
          orderbook1: {
            address: '0xOrderbookAddress1',
            network: 'mainnet',
            subgraph: 'mainnet',
            label: 'Orderbook 1',
          },
          orderbook2: {
            address: '0xOrderbookAddress2',
            network: 'testnet',
            subgraph: 'testnet',
            label: 'Orderbook 2',
          },
        },
      };

      settings.set(multiNetworkConfig);
      activeNetworkRef.set('mainnet');

      const filteredOrderbooks = get(activeNetworkOrderbooks);
      expect(filteredOrderbooks).toEqual({
        orderbook1: multiNetworkConfig.orderbooks.orderbook1,
      });
    });

    test('should reset orderbook when network changes to incompatible one', () => {
      const multiNetworkConfig = {
        ...mockConfigSource,
        networks: {
          mainnet: { rpc: 'mainnet.rpc', 'chain-id': 1 },
          testnet: { rpc: 'testnet.rpc', 'chain-id': 5 },
        },
        orderbooks: {
          orderbook1: {
            address: '0xOrderbookAddress1',
            network: 'mainnet',
            subgraph: 'mainnet',
            label: 'Orderbook 1',
          },
        },
      };

      settings.set(multiNetworkConfig);
      activeNetworkRef.set('mainnet');
      activeOrderbookRef.set('orderbook1');

      activeNetworkRef.set('testnet');

      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('resetActiveOrderbookRef should set first available orderbook', () => {
      settings.set(mockConfigSource);
      activeNetworkRef.set('mainnet');

      resetActiveOrderbookRef();

      expect(get(activeOrderbookRef)).toBe('orderbook1');
    });

    test('resetActiveOrderbookRef should set undefined when no orderbooks', () => {
      settings.set(mockConfigSource);
      activeNetworkRef.set('mainnet');

      const newSettings = {
        ...mockConfigSource,
        orderbooks: {},
      };
      settings.set(newSettings);

      resetActiveOrderbookRef();

      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('hasRequiredSettings should return true when both refs are set', () => {
      activeNetworkRef.set('mainnet');
      activeOrderbookRef.set('orderbook1');

      expect(get(hasRequiredSettings)).toBe(true);
    });

    test('hasRequiredSettings should return false when refs are missing', () => {
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set('orderbook1');

      expect(get(hasRequiredSettings)).toBe(false);
    });
  });

  describe('Derived Store Behaviors', () => {
    beforeEach(() => {
      settings.set(undefined);
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set(undefined);
    });

    test('subgraphUrl should return undefined when no settings', () => {
      expect(get(subgraphUrl)).toBeUndefined();
    });

    test('subgraphUrl should derive correctly when settings available', () => {
      settings.set({
        ...mockConfigSource,
        subgraphs: {
          mainnet: 'https://api.thegraph.com/subgraphs/name/mainnet',
        },
        orderbooks: {
          orderbook1: {
            address: '0xOrderbookAddress1',
            network: 'mainnet',
            subgraph: 'mainnet',
          },
        },
      });
      activeOrderbookRef.set('orderbook1');

      expect(get(subgraphUrl)).toBe('https://api.thegraph.com/subgraphs/name/mainnet');
    });

    test('accounts should return empty object when no settings', () => {
      expect(get(accounts)).toEqual({});
    });

    test('activeAccounts should filter based on activeAccountsItems', () => {
      settings.set(mockConfigSource);
      activeAccountsItems.set({
        name_one: 'address_one',
      });

      expect(get(activeAccounts)).toEqual({
        name_one: 'address_one',
      });
    });

    test('activeAccounts should return empty when activeAccountsItems is empty', () => {
      settings.set(mockConfigSource);
      activeAccountsItems.set({});

      expect(get(activeAccounts)).toEqual({});
    });
  });

  describe('Additional Store Functionality', () => {
    test('activeOrderStatus should properly serialize/deserialize boolean values', () => {
      activeOrderStatus.set(true);
      expect(get(activeOrderStatus)).toBe(true);

      activeOrderStatus.set(false);
      expect(get(activeOrderStatus)).toBe(false);

      activeOrderStatus.set(undefined);
      expect(get(activeOrderStatus)).toBeUndefined();
    });

    test('hideZeroBalanceVaults should default to true', () => {
      expect(get(hideZeroBalanceVaults)).toBe(true);
    });

    test('hideZeroBalanceVaults should handle true/false values', () => {
      hideZeroBalanceVaults.set(false);
      expect(get(hideZeroBalanceVaults)).toBe(false);

      hideZeroBalanceVaults.set(true);
      expect(get(hideZeroBalanceVaults)).toBe(true);
    });

    test('orderHash should handle string values correctly', () => {
      orderHash.set('test-hash');
      expect(get(orderHash)).toBe('test-hash');

      orderHash.set('');
      expect(get(orderHash)).toBe('');
    });
  });

  describe('Settings Subscription Edge Cases', () => {
    test('should handle invalid JSON in settings', () => {
      const settingsWithBreak = cachedWritableStore<ConfigSource | undefined>(
        'settings-test',
        undefined,
        (value) => (value ? JSON.stringify(value) : 'invalid-json'),
        (str) => {
          try {
            return JSON.parse(str) as ConfigSource;
          } catch {
            return undefined;
          }
        },
      );

      settingsWithBreak.set(mockConfigSource);
      expect(get(settingsWithBreak)).toEqual(mockConfigSource);
    });

    test('should handle account items with extra accounts', () => {
      settings.set(mockConfigSource);
      activeAccountsItems.set({
        name_one: 'address_one',
        name_two: 'address_two',
        name_three: 'address_three',
      });

      settings.set(mockConfigSource);

      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
        name_two: 'address_two',
      });
    });
  });
}
