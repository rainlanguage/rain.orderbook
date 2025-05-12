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
  subgraphUrl,
  accounts,
  activeAccounts,
} from '$lib/stores/settings';
import { mockConfigSource } from '$lib/mocks/mockConfigSource';
import { beforeEach, describe } from 'vitest';
import { get } from '@square/svelte-store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import type { ConfigSource } from '@rainlanguage/orderbook';

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
      settingsWithBreak.set(mockConfigSource);
      expect(get(settingsWithBreak)).toEqual(mockConfigSource);
    });

    test('should handle account items with extra accounts not in settings', () => {
      settings.set(mockConfigSource); // settings.accounts has name_one, name_two
      activeAccountsItems.set({
        name_one: 'address_one', // Will be kept
        name_two: 'address_two', // Will be kept
        name_three: 'address_three', // Should be removed by the subscription
      });

      // Trigger the subscription by setting settings again (even if it's the same)
      // The subscription logic will re-evaluate activeAccountsItems
      settings.set({ ...mockConfigSource });

      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
        name_two: 'address_two',
      });
    });
  });
}
