import {
  settings,
  activeAccountsItems,
  accounts,
  activeAccounts,
  EMPTY_SETTINGS,
  selectedChainIds,
} from '$lib/stores/settings';
import { mockConfig } from '$lib/mocks/mockConfig';
import { beforeEach, describe, expect, test } from 'vitest';
import { get } from '@square/svelte-store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import type { ConfigSource, NewConfig } from '@rainlanguage/orderbook';

describe('Settings active accounts items', () => {
  // Reset store values before each test to prevent state leakage
  beforeEach(() => {
    settings.set(EMPTY_SETTINGS);
    activeAccountsItems.set({});
    selectedChainIds.set([]);

    settings.set(mockConfig);
    activeAccountsItems.set({
      name_one: '0xaddress_one',
      name_two: '0xaddress_two',
    });
    selectedChainIds.set([1, 2]);

    // Verify initial state
    expect(get(settings)).toEqual(mockConfig);
    expect(get(activeAccountsItems)).toEqual({
      name_one: '0xaddress_one',
      name_two: '0xaddress_two',
    });
    expect(get(selectedChainIds)).toEqual([1, 2]);
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
            address: '0xaddress_one',
          },
        },
      },
    };

    // Update settings - this should trigger the subscription
    settings.set(newSettings as unknown as NewConfig);

    // Check the expected result
    expect(get(activeAccountsItems)).toEqual({
      name_one: '0xaddress_one',
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
            address: '0xaddress_one',
          },
          name_two: {
            key: 'name_two',
            address: '0xnew_value',
          },
        },
      },
    };

    settings.set(newSettings as unknown as NewConfig);

    expect(get(activeAccountsItems)).toEqual({
      name_one: '0xaddress_one',
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

    settings.set(newSettings as unknown as NewConfig);

    expect(get(activeAccountsItems)).toEqual({});
  });
});

describe('Derived Store Behaviors', () => {
  beforeEach(() => {
    settings.set(EMPTY_SETTINGS);
  });

  test('accounts should return empty object when no settings', () => {
    expect(get(accounts)).toEqual({});
  });

  test('activeAccounts should filter based on activeAccountsItems', () => {
    settings.set(mockConfig);
    activeAccountsItems.set({
      name_one: '0xaddress_one',
    });

    expect(get(activeAccounts)).toEqual({
      name_one: {
        key: 'name_one',
        address: '0xaddress_one',
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
      name_one: '0xaddress_one', // Will be kept
      name_two: '0xaddress_two', // Will be kept
      name_three: '0xaddress_three', // Should be removed by the subscription
    });

    // Trigger the subscription by setting settings again (even if it's the same)
    // The subscription logic will re-evaluate activeAccountsItems
    settings.set({ ...mockConfig });

    expect(get(activeAccountsItems)).toEqual({
      name_one: '0xaddress_one',
      name_two: '0xaddress_two',
    });
  });
});
