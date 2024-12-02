import { expect, test, beforeEach, describe } from 'vitest';
import { settings, activeAccountsItems, activeSubgraphs } from './settings';
import { mockConfigSource } from '@rainlanguage/ui-components';
import { get } from 'svelte/store';

describe('Settings active accounts items', async () => {
  beforeEach(() => {
    activeSubgraphs.set({
      mainnet: 'mainnet',
    });
    activeAccountsItems.set(mockConfigSource.accounts as Record<string, string>);
    expect(get(activeAccountsItems)).toEqual(mockConfigSource.accounts);
  });

  test('should remove account if that account is removed', () => {
    const newSettings = {
      ...mockConfigSource,
      accounts: {
        name_one: mockConfigSource.accounts?.name_one as string,
      },
    };

    settings.set(newSettings);

    expect(get(activeAccountsItems)).toEqual({
      name_one: mockConfigSource.accounts?.name_one as string,
    });
  });

  test('should remove account if the value is different', () => {
    const newSettings = {
      ...mockConfigSource,
      accounts: {
        name_one: mockConfigSource.accounts?.name_one as string,
        name_two: 'new_value',
      },
    };

    settings.set(newSettings);

    expect(get(activeAccountsItems)).toEqual({
      name_one: mockConfigSource.accounts?.name_one as string,
    });
  });

  test('should update active subgraphs when subgraph value change', () => {
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
    settings.set({
      ...mockConfigSource,
      subgraphs: undefined,
    });
    expect(get(activeSubgraphs)).toEqual({});
  });
});
