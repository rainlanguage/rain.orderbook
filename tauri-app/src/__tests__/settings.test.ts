import {
  settings,
  activeAccountsItems,
  activeSubgraphs,
  activeNetworkRef,
  activeOrderbookRef,
  resetActiveNetworkRef,
  resetActiveOrderbookRef,
  activeNetworkOrderbooks,
  hasRequiredSettings,
  subgraph,
  accounts,
  activeAccounts,
  EMPTY_SETTINGS,
} from '$lib/stores/settings';
import { mockConfig } from '$lib/mocks/mockConfig';
import { beforeEach, describe, expect, test } from 'vitest';
import { get } from '@square/svelte-store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import type { ConfigSource, Config } from '@rainlanguage/orderbook';

describe('Settings active accounts items', () => {
  // Reset store values before each test to prevent state leakage
  beforeEach(() => {
    settings.set(EMPTY_SETTINGS);
    activeAccountsItems.set({});
    activeSubgraphs.set({});
    activeNetworkRef.set(undefined);
    activeOrderbookRef.set(undefined);

    settings.set(mockConfig);
    activeAccountsItems.set({
      name_one: 'address_one',
      name_two: 'address_two',
    });
    activeSubgraphs.set({
      mainnet: {
        key: 'mainnet',
        url: 'https://api.thegraph.com/subgraphs/name/mainnet',
      },
    });

    // Verify initial state
    expect(get(settings)).toEqual(mockConfig);
    expect(get(activeAccountsItems)).toEqual({
      name_one: 'address_one',
      name_two: 'address_two',
    });
    expect(get(activeSubgraphs)).toEqual({
      mainnet: {
        key: 'mainnet',
        url: 'https://api.thegraph.com/subgraphs/name/mainnet',
      },
    });
  });

  test('should remove account if that account is removed', () => {
    // Test removing an account
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        accounts: {
          name_one: {
            key: 'name_one',
            address: 'address_one',
          },
        },
      },
    };

    // Update settings - this should trigger the subscription
    settings.set(newSettings as unknown as Config);

    // Check the expected result
    expect(get(activeAccountsItems)).toEqual({
      name_one: 'address_one',
    });
  });

  test('should remove account if the value is different', () => {
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        accounts: {
          name_one: {
            key: 'name_one',
            address: 'address_one',
          },
          name_two: {
            key: 'name_two',
            address: 'new_value',
          },
        },
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeAccountsItems)).toEqual({
      name_one: 'address_one',
    });
  });

  test('should reset active accounts when accounts are empty', () => {
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        accounts: {},
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeAccountsItems)).toEqual({});
  });

  test('should update active subgraphs when subgraph value changes', () => {
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        subgraphs: {
          mainnet: {
            key: 'mainnet',
            url: 'new value',
          },
        },
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeSubgraphs)).toEqual({});
  });

  test('should update active subgraphs when subgraph removed', () => {
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        subgraphs: {
          testnet: {
            key: 'testnet',
            url: 'testnet',
          },
        },
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeSubgraphs)).toEqual({});
  });

  test('should reset active subgraphs when subgraphs are empty', () => {
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        subgraphs: {},
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeSubgraphs)).toEqual({});
  });
});

describe('Network and Orderbook Management', () => {
  beforeEach(() => {
    // Reset all stores
    settings.set(EMPTY_SETTINGS);
    activeNetworkRef.set(undefined);
    activeOrderbookRef.set(undefined);
    activeAccountsItems.set({});
    activeSubgraphs.set({});
  });

  test('should reset activeNetworkRef when networks are empty', () => {
    // First set valid settings
    settings.set(mockConfig);
    activeNetworkRef.set('mainnet');

    // Then make networks empty
    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        networks: {},
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeNetworkRef)).toBeUndefined();
  });

  test('should reset activeOrderbookRef when activeNetworkRef is undefined', () => {
    settings.set(mockConfig);
    activeNetworkRef.set('mainnet');
    activeOrderbookRef.set('orderbook1');

    activeNetworkRef.set(undefined);

    expect(get(activeOrderbookRef)).toBeUndefined();
  });

  test('resetActiveNetworkRef should set first available network', async () => {
    settings.set(mockConfig);

    resetActiveNetworkRef();

    expect(get(activeNetworkRef)).toBe('mainnet');
  });

  test('resetActiveNetworkRef should set undefined when no networks', async () => {
    const emptySettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        networks: {},
      },
    };
    settings.set(emptySettings as unknown as Config);

    resetActiveNetworkRef();

    expect(get(activeNetworkRef)).toBeUndefined();
  });

  test('should reset activeOrderbookRef when orderbooks are empty', () => {
    settings.set(mockConfig);
    activeOrderbookRef.set('orderbook1');

    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        orderbooks: {},
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeOrderbookRef)).toBeUndefined();
  });

  test('should filter orderbooks by active network', () => {
    const multiNetworkConfig = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        orderbooks: {
          orderbook1: {
            address: '0xOrderbookAddress1',
            network: {
              key: 'mainnet',
              rpc: 'mainnet.rpc',
              chainId: 1,
            },
            subgraph: {
              key: 'mainnet',
              url: 'mainnet',
            },
            label: 'Orderbook 1',
          },
          orderbook2: {
            address: '0xOrderbookAddress2',
            network: {
              key: 'testnet',
              rpc: 'testnet.rpc',
              chainId: 5,
            },
            subgraph: {
              key: 'testnet',
              url: 'testnet',
            },
            label: 'Orderbook 2',
          },
        },
      },
    };

    settings.set(multiNetworkConfig as unknown as Config);
    activeNetworkRef.set('mainnet');

    const filteredOrderbooks = get(activeNetworkOrderbooks);
    expect(filteredOrderbooks).toEqual({
      orderbook1: multiNetworkConfig.orderbook.orderbooks.orderbook1,
    });
  });

  test('should reset orderbook when network changes to incompatible one', () => {
    const multiNetworkConfig = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        networks: {
          mainnet: { key: 'mainnet', rpc: 'mainnet.rpc', chainId: 1 },
          testnet: { key: 'mainnet', rpc: 'testnet.rpc', chainId: 5 },
        },
        orderbooks: {
          orderbook1: {
            address: '0xOrderbookAddress1',
            network: {
              key: 'mainnet',
              rpc: 'mainnet.rpc',
              chainId: 1,
            },
            subgraph: {
              key: 'mainnet',
              url: 'mainnet',
            },
            label: 'Orderbook 1',
          },
        },
      },
    };

    settings.set(multiNetworkConfig as unknown as Config);
    activeNetworkRef.set('mainnet');
    activeOrderbookRef.set('orderbook1');

    activeNetworkRef.set('testnet');

    expect(get(activeOrderbookRef)).toBeUndefined();
  });

  test('resetActiveOrderbookRef should set first available orderbook', () => {
    settings.set(mockConfig);
    activeNetworkRef.set('mainnet');

    resetActiveOrderbookRef();

    expect(get(activeOrderbookRef)).toBe('orderbook1');
  });

  test('resetActiveOrderbookRef should set undefined when no orderbooks', () => {
    settings.set(mockConfig);
    activeNetworkRef.set('mainnet');

    const newSettings = {
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        orderbooks: {},
      },
    };
    settings.set(newSettings as unknown as Config);

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
    settings.set(EMPTY_SETTINGS);
    activeNetworkRef.set(undefined);
    activeOrderbookRef.set(undefined);
  });

  test('subgraphUrl should return undefined when no settings', () => {
    expect(get(subgraph)).toBeUndefined();
  });

  test('subgraph should derive correctly when settings available', () => {
    settings.set({
      ...mockConfig,
      orderbook: {
        ...mockConfig.orderbook,
        subgraphs: {
          mainnet: {
            key: 'mainnet',
            url: 'https://api.thegraph.com/subgraphs/name/mainnet',
          },
        },
        orderbooks: {
          orderbook1: {
            address: '0xOrderbookAddress1',
            network: {
              key: 'mainnet',
              rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
              chainId: 1,
            },
            subgraph: {
              key: 'mainnet',
              url: 'https://api.thegraph.com/subgraphs/name/mainnet',
            },
          },
        },
      },
    } as unknown as Config);
    activeOrderbookRef.set('orderbook1');

    expect(get(subgraph)).toStrictEqual({
      key: 'mainnet',
      url: 'https://api.thegraph.com/subgraphs/name/mainnet',
    });
  });

  test('accounts should return empty object when no settings', () => {
    expect(get(accounts)).toEqual({});
  });

  test('activeAccounts should filter based on activeAccountsItems', () => {
    settings.set(mockConfig);
    activeAccountsItems.set({
      name_one: 'address_one',
    });

    expect(get(activeAccounts)).toEqual({
      name_one: {
        key: 'name_one',
        address: 'address_one',
      },
    });
  });

  test('activeAccounts should return empty when activeAccountsItems is empty', () => {
    settings.set(mockConfig);
    activeAccountsItems.set({});

    expect(get(activeAccounts)).toEqual({});
  });
});

describe('Settings Subscription Edge Cases', () => {
  test('should handle invalid JSON in settings', () => {
    // This test uses a local store to avoid interfering with the global 'settings' store
    // and to directly test the JSON parsing logic of cachedWritableStore.
    const settingsWithBreak = cachedWritableStore<ConfigSource | undefined>(
      'settings-test-invalid-json', // Unique key
      undefined,
      (value) => (value ? JSON.stringify(value) : 'invalid-json'), // Force invalid JSON string for undefined
      (str) => {
        try {
          return JSON.parse(str) as ConfigSource;
        } catch {
          return undefined;
        }
      },
    );
    // Try to save 'undefined', which gets serialized to 'invalid-json'
    settingsWithBreak.set(undefined);
    // Then try to get it, which should trigger the catch and return undefined
    expect(get(settingsWithBreak)).toBeUndefined();

    // Test with valid data
    settingsWithBreak.set(mockConfig as unknown as ConfigSource);
    expect(get(settingsWithBreak)).toEqual(mockConfig);
  });

  test('should handle account items with extra accounts not in settings', () => {
    settings.set(mockConfig); // settings.accounts has name_one, name_two
    activeAccountsItems.set({
      name_one: 'address_one', // Will be kept
      name_two: 'address_two', // Will be kept
      name_three: 'address_three', // Should be removed by the subscription
    });

    // Trigger the subscription by setting settings again (even if it's the same)
    // The subscription logic will re-evaluate activeAccountsItems
    settings.set({ ...mockConfig });

    expect(get(activeAccountsItems)).toEqual({
      name_one: 'address_one',
      name_two: 'address_two',
    });
  });
});
