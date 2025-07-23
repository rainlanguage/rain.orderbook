import { activeAccountsItems, selectedChainIds } from '$lib/stores/settings';
import { mockConfig } from '$lib/mocks/mockConfig';
import { beforeEach, describe, expect, test } from 'vitest';
import { get } from '@square/svelte-store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import type { ConfigSource } from '@rainlanguage/orderbook';

describe('Settings active accounts items', () => {
  // Reset store values before each test to prevent state leakage
  beforeEach(() => {
    activeAccountsItems.set({});
    selectedChainIds.set([]);

    activeAccountsItems.set({
      name_one: '0xaddress_one',
      name_two: '0xaddress_two',
    });
    selectedChainIds.set([1, 2]);

    // Verify initial state
    expect(get(activeAccountsItems)).toEqual({
      name_one: '0xaddress_one',
      name_two: '0xaddress_two',
    });
    expect(get(selectedChainIds)).toEqual([1, 2]);
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
});
