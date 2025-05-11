import { activeSubgraphs } from '$lib/stores/settings';
import { activeAccountsItems } from '$lib/stores/settings';
import { mockConfigSource } from '$lib/mocks/mockConfigSource';
import { settings } from '$lib/stores/settings';
import { beforeEach, describe } from 'vitest';
import { get } from '@square/svelte-store';

if (import.meta.vitest) {
  const { test, expect } = import.meta.vitest;

  describe('Settings active accounts items', () => {
    // Reset store values before each test to prevent state leakage
    beforeEach(() => {
      // Reset all store values
      settings.set(undefined);
      activeAccountsItems.set({});
      activeSubgraphs.set({});

      // Then set our initial test values
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
}
