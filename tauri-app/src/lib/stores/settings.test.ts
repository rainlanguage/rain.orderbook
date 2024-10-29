import { expect, test, beforeEach, describe } from 'vitest';
import { settings, activeAccountsItems } from './settings';
import { mockConfigSource } from '$lib/mocks/settings';
import { get } from 'svelte/store';

describe('Settings active accounts items', async () => {
  beforeEach(() => {
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
});
