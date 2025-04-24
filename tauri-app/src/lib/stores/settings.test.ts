import { expect, test, beforeEach, describe } from 'vitest';
import { settings, activeAccountsItems, activeSubgraphs } from './settings';
import { get } from 'svelte/store';
import type { Config, NetworkCfg, SubgraphCfg } from '@rainlanguage/orderbook';

// Define the mock directly in the tests
const mockConfig: Config = {
  networks: {
    mainnet: {
      key: 'mainnet',
      rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
      chainId: 1,
      label: 'Ethereum Mainnet',
      currency: 'ETH',
    },
  },
  subgraphs: {
    mainnet: {
      key: 'mainnet',
      url: 'https://api.thegraph.com/subgraphs/name/mainnet',
    },
  },
  orderbooks: {
    orderbook1: {
      key: 'orderbook1',
      address: '0xOrderbookAddress1',
      network: {
        key: 'mainnet',
      } as unknown as NetworkCfg,
      subgraph: {
        key: 'uniswap',
      } as unknown as SubgraphCfg,
      label: 'Orderbook 1',
    },
  },
  deployers: {
    deployer1: {
      key: 'deployer1',
      address: '0xDeployerAddress1',
      network: {
        key: 'mainnet',
      } as unknown as NetworkCfg,
    },
  },
  metaboards: {
    metaboard1: 'https://example.com/metaboard1',
  },
  accounts: {
    name_one: 'address_one',
    name_two: 'address_two',
  },
} as unknown as Config;

describe('Settings active accounts items', () => {
  // Reset store values before each test to prevent state leakage
  beforeEach(() => {
    // Reset all store values
    settings.set(undefined);
    activeAccountsItems.set({});
    activeSubgraphs.set({});

    // Then set our initial test values
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
      accounts: {
        name_one: {
          key: 'name_one',
          address: 'address_one',
        },
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
      ...mockConfig,
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
    };

    settings.set(newSettings);

    expect(get(activeAccountsItems)).toEqual({
      name_one: 'address_one',
    });
  });

  test('should update active subgraphs when subgraph value changes', () => {
    const newSettings = {
      ...mockConfig,
      subgraphs: {
        mainnet: {
          key: 'mainnet',
          url: 'new value',
        },
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeSubgraphs)).toEqual({});
  });

  test('should update active subgraphs when subgraph removed', () => {
    const newSettings = {
      ...mockConfig,
      subgraphs: {
        testnet: {
          key: 'testnet',
          url: 'testnet',
        },
      },
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeSubgraphs)).toEqual({});
  });

  test('should reset active subgraphs when subgraphs are undefined', () => {
    const newSettings = {
      ...mockConfig,
      subgraphs: undefined,
    };

    settings.set(newSettings as unknown as Config);

    expect(get(activeSubgraphs)).toEqual({});
  });
});
