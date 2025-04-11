import { expect, test, beforeEach, describe } from 'vitest';
import { settings, activeAccountsItems, activeSubgraphs } from './settings';
import { get } from 'svelte/store';

// Import the ConfigSource type
import type { ConfigSource } from '@rainlanguage/orderbook';

// Define the mock directly in the tests
const mockConfigSource: ConfigSource = {
  networks: {
    mainnet: {
      rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
      'chain-id': 1,
      label: 'Ethereum Mainnet',
      currency: 'ETH',
    },
  },
  subgraphs: {
    mainnet: 'https://api.thegraph.com/subgraphs/name/mainnet',
  },
  orderbooks: {
    orderbook1: {
      address: '0xOrderbookAddress1',
      network: 'mainnet',
      subgraph: 'uniswap',
      label: 'Orderbook 1',
    },
  },
  deployers: {
    deployer1: {
      address: '0xDeployerAddress1',
      network: 'mainnet',
      label: 'Deployer 1',
    },
  },
  metaboards: {
    metaboard1: 'https://example.com/metaboard1',
  },
  accounts: {
    name_one: 'address_one',
    name_two: 'address_two',
  },
};

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
