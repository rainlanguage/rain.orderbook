import { derived, get } from '@square/svelte-store';
import { cachedWritableStore, cachedWritableStringOptional } from '@rainlanguage/ui-components';
import find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import {
  parseYaml,
  type Config,
  type OrderbookCfg,
  type SubgraphCfg,
} from '@rainlanguage/orderbook';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { pickBy } from 'lodash';
import { waitFor } from '@testing-library/svelte';

export const EMPTY_SETTINGS: Config = {
  orderbook: {
    networks: {},
    subgraphs: {},
    metaboards: {},
    orderbooks: {},
    accounts: {},
    tokens: {},
    deployers: {},
  },
  dotrainOrder: {
    orders: {},
    scenarios: {},
    charts: {},
    deployments: {},
  },
};

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
export const settings = cachedWritableStore<Config>(
  'settings',
  EMPTY_SETTINGS,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      return parseYaml([str]);
    } catch {
      return EMPTY_SETTINGS;
    }
  },
);
export const enableSentry = derived(settings, ($settings) =>
  $settings.orderbook.sentry !== undefined ? $settings.orderbook.sentry : true,
);

// networks
export const activeNetworkRef = cachedWritableStringOptional('settings.activeNetworkRef');
export const activeNetwork = derived(
  [settings, activeNetworkRef],
  ([$settings, $activeNetworkRef]) => {
    return $activeNetworkRef !== undefined && $settings.orderbook.networks !== undefined
      ? $settings.orderbook.networks[$activeNetworkRef]
      : undefined;
  },
);
export const rpcUrl = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.rpc);
export const chainId = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.chainId);
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
    $settings.orderbook.orderbooks
      ? (pickBy(
          $settings.orderbook.orderbooks,
          (orderbook) => orderbook.network.key === $activeNetworkRef,
        ) as Record<string, OrderbookCfg>)
      : ({} as Record<string, OrderbookCfg>),
);
export const activeOrderbook = derived(
  [settings, activeOrderbookRef],
  ([$settings, $activeOrderbookRef]) =>
    $settings.orderbook.orderbooks !== undefined && $activeOrderbookRef !== undefined
      ? $settings.orderbook.orderbooks[$activeOrderbookRef]
      : undefined,
);
export const subgraph = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
  $settings.orderbook.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
    ? $settings.orderbook.subgraphs[$activeOrderbook.subgraph.key]
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
export const accounts = derived(settings, ($settings) => $settings.orderbook.accounts ?? {});
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
export const subgraphs = derived(settings, ($settings) =>
  $settings.orderbook.subgraphs !== undefined ? Object.entries($settings.orderbook.subgraphs) : [],
);
export const activeSubgraphs = cachedWritableStore<Record<string, SubgraphCfg>>(
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
    $settings.orderbook.networks === undefined ||
    $activeNetworkRef === undefined ||
    ($settings.orderbook.networks !== undefined &&
      $activeNetworkRef !== undefined &&
      !Object.keys($settings.orderbook.networks).includes($activeNetworkRef))
  ) {
    resetActiveNetworkRef();
  }

  if (
    $settings.orderbook.orderbooks === undefined ||
    $activeOrderbookRef === undefined ||
    ($settings.orderbook.orderbooks !== undefined &&
      $activeOrderbookRef !== undefined &&
      !Object.keys($settings.orderbook.orderbooks).includes($activeOrderbookRef))
  ) {
    resetActiveOrderbookRef();
  }

  // Reset active account items if accounts have changed
  if ($settings.orderbook.accounts === undefined) {
    activeAccountsItems.set({});
  } else {
    const currentActiveAccounts = get(activeAccountsItems);
    const updatedActiveAccounts = Object.fromEntries(
      Object.entries($settings.orderbook.accounts ?? {})
        .filter(([key, value]) => {
          if (key in currentActiveAccounts) {
            return currentActiveAccounts[key] === value.address;
          }
          return false;
        })
        .map(([key, value]) => [key, value.address]),
    );
    activeAccountsItems.set(updatedActiveAccounts);
  }

  // Reset active subgraphs if subgraphs have changed
  if ($settings.orderbook.subgraphs === undefined) {
    activeSubgraphs.set({});
  } else {
    const currentActiveSubgraphs = get(activeSubgraphs);
    const updatedActiveSubgraphs = Object.fromEntries(
      Object.entries($settings.orderbook.subgraphs).filter(([key, value]) => {
        if (key in currentActiveSubgraphs) {
          return JSON.stringify(currentActiveSubgraphs[key]) === JSON.stringify(value);
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
  const $networks = get(settings)?.orderbook.networks;

  if ($networks !== undefined && Object.keys($networks).length > 0) {
    activeNetworkRef.set(Object.keys($networks)[0]);
  } else {
    activeNetworkRef.set(undefined);
  }
}

export const showInactiveOrders = cachedWritableStore<boolean>(
  'settings.showInactiveOrders',
  true,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      const value = JSON.parse(str);
      return typeof value === 'boolean' ? value : true;
    } catch {
      return true;
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
  const { beforeEach, describe, test, expect } = import.meta.vitest;

  const mockConfig: Config = {
    orderbook: {
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
            rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
            chainId: 1,
            label: 'Ethereum Mainnet',
            currency: 'ETH',
          },
          subgraph: {
            key: 'mainnet',
            url: 'https://api.thegraph.com/subgraphs/name/mainnet',
          },
        },
      },
      tokens: {},
      deployers: {
        deployer1: {
          key: 'deployer1',
          address: '0xDeployerAddress1',
          network: {
            key: 'mainnet',
            rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
            chainId: 1,
            label: 'Ethereum Mainnet',
            currency: 'ETH',
          },
        },
      },
      metaboards: {
        metaboard1: 'https://example.com/metaboard1',
      },
      accounts: {
        name_one: {
          key: 'name_one',
          address: 'address_one',
        },
        name_two: {
          key: 'name_two',
          address: 'address_two',
        },
      },
    },
    dotrainOrder: {
      orders: {},
      scenarios: {},
      charts: {},
      deployments: {},
    },
  };

  describe('Settings active accounts items', () => {
    // Reset store values before each test to prevent state leakage
    beforeEach(() => {
      // Reset all store values
      settings.set(EMPTY_SETTINGS);
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
        orderbook: {
          ...mockConfig.orderbook,
          accounts: {
            name_one: {
              key: 'name_one',
              address: 'address_one',
            },
          },
        },
      } as unknown as Config;

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
      } as unknown as Config;

      settings.set(newSettings);

      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
      });
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
      } as unknown as Config;

      settings.set(newSettings);

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
      } as unknown as Config;

      settings.set(newSettings);

      expect(get(activeSubgraphs)).toEqual({});
    });

    test('should reset active subgraphs when subgraphs are undefined', () => {
      const newSettings = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          subgraphs: undefined,
        },
      } as unknown as Config;

      settings.set(newSettings);

      expect(get(activeSubgraphs)).toEqual({});
    });
  });

  // Everything below this line should be updated to match the settings pattern used above.
  // DO NOT edit anything above this line

  describe('Network and Orderbook Management', () => {
    beforeEach(() => {
      // Reset all store values before each test to prevent state leakage
      settings.set(EMPTY_SETTINGS);
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set(undefined);
      activeAccountsItems.set({});
      activeSubgraphs.set({});

      // Set initial test values
      settings.set(mockConfig);
    });

    test('should reset activeNetworkRef when networks are undefined', () => {
      // First set valid network reference
      activeNetworkRef.set('mainnet');

      // Then make networks undefined
      const newSettings = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          networks: undefined,
        },
      } as unknown as Config;

      // Update settings - this should trigger the subscription
      settings.set(newSettings);

      // Check the expected result
      expect(get(activeNetworkRef)).toBeUndefined();
    });

    test('should reset activeOrderbookRef when activeNetworkRef is undefined', () => {
      // Setup initial state
      activeNetworkRef.set('mainnet');
      activeOrderbookRef.set('orderbook1');

      // Trigger the test condition
      activeNetworkRef.set(undefined);

      // Check the expected result
      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('resetActiveNetworkRef should set first available network', async () => {
      // Reset activeNetworkRef to undefined first
      activeNetworkRef.set(undefined);

      // Call the function being tested
      await resetActiveNetworkRef();

      // Check the expected result
      expect(get(activeNetworkRef)).toBe('mainnet');
    });

    test('resetActiveNetworkRef should set undefined when no networks', async () => {
      // Create empty networks in settings
      const emptyNetworksSettings = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          networks: {},
        },
      } as unknown as Config;
      settings.set(emptyNetworksSettings);

      // Call the function being tested
      await resetActiveNetworkRef();

      // Check the expected result
      expect(get(activeNetworkRef)).toBeUndefined();
    });

    test('should reset activeOrderbookRef when orderbooks are undefined', () => {
      // Setup initial state
      activeOrderbookRef.set('orderbook1');

      // Create test settings with undefined orderbooks
      const newSettings = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          orderbooks: undefined,
        },
      } as unknown as Config;

      // Update settings - this should trigger the subscription
      settings.set(newSettings);

      // Check the expected result
      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('should filter orderbooks by active network', () => {
      // Create multi-network config for testing
      const multiNetworkConfig = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          networks: {
            mainnet: {
              key: 'mainnet',
              rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
              chainId: 1,
              label: 'Ethereum Mainnet',
              currency: 'ETH',
            },
            testnet: {
              key: 'testnet',
              rpc: 'https://testnet.infura.io/v3/YOUR-PROJECT-ID',
              chainId: 5,
              label: 'Ethereum Testnet',
              currency: 'ETH',
            },
          },
          orderbooks: {
            orderbook1: {
              key: 'orderbook1',
              address: '0xOrderbookAddress1',
              network: {
                key: 'mainnet',
              },
              subgraph: {
                key: 'mainnet',
              },
              label: 'Orderbook 1',
            },
            orderbook2: {
              key: 'orderbook2',
              address: '0xOrderbookAddress2',
              network: {
                key: 'testnet',
              },
              subgraph: {
                key: 'testnet',
              },
              label: 'Orderbook 2',
            },
          },
        },
      } as unknown as Config;

      // Set the multi-network config
      settings.set(multiNetworkConfig);

      // Set active network to mainnet
      activeNetworkRef.set('mainnet');

      // Get the filtered orderbooks
      const filteredOrderbooks = get(activeNetworkOrderbooks);

      // Check that only the mainnet orderbook is included
      expect(Object.keys(filteredOrderbooks)).toEqual(['orderbook1']);
    });

    test('should reset orderbook when network changes to incompatible one', () => {
      // Create multi-network config for testing
      const multiNetworkConfig = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          networks: {
            mainnet: {
              key: 'mainnet',
              rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
              chainId: 1,
              label: 'Ethereum Mainnet',
              currency: 'ETH',
            },
            testnet: {
              key: 'testnet',
              rpc: 'https://testnet.infura.io/v3/YOUR-PROJECT-ID',
              chainId: 5,
              label: 'Ethereum Testnet',
              currency: 'ETH',
            },
          },
          orderbooks: {
            orderbook1: {
              key: 'orderbook1',
              address: '0xOrderbookAddress1',
              network: {
                key: 'mainnet',
              },
              subgraph: {
                key: 'mainnet',
              },
              label: 'Orderbook 1',
            },
          },
        },
      } as unknown as Config;

      // Set the multi-network config
      settings.set(multiNetworkConfig);

      // Set up initial state
      activeNetworkRef.set('mainnet');
      activeOrderbookRef.set('orderbook1');

      // Change to incompatible network
      activeNetworkRef.set('testnet');

      // Check the expected result
      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('resetActiveOrderbookRef should set first available orderbook', () => {
      // Set network to mainnet
      activeNetworkRef.set('mainnet');

      // Reset active orderbook
      resetActiveOrderbookRef();

      // Check the expected result
      expect(get(activeOrderbookRef)).toBe('orderbook1');
    });

    test('resetActiveOrderbookRef should set undefined when no orderbooks', () => {
      // Set network to mainnet
      activeNetworkRef.set('mainnet');

      // Create settings with empty orderbooks
      const newSettings = {
        ...mockConfig,
        orderbook: {
          ...mockConfig.orderbook,
          orderbooks: {},
        },
      } as unknown as Config;

      // Update settings
      settings.set(newSettings);

      // Reset active orderbook
      resetActiveOrderbookRef();

      // Check the expected result
      expect(get(activeOrderbookRef)).toBeUndefined();
    });

    test('hasRequiredSettings should return true when both refs are set', () => {
      // Set both network and orderbook references
      activeNetworkRef.set('mainnet');
      activeOrderbookRef.set('orderbook1');

      // Check the expected result
      expect(get(hasRequiredSettings)).toBe(true);
    });

    test('hasRequiredSettings should return false when refs are missing', () => {
      // Set orderbook ref but not network ref
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set('orderbook1');

      // Check the expected result
      expect(get(hasRequiredSettings)).toBe(false);
    });
  });

  describe('Derived Store Behaviors', () => {
    beforeEach(() => {
      // Reset all store values
      settings.set(EMPTY_SETTINGS);
      activeNetworkRef.set(undefined);
      activeOrderbookRef.set(undefined);
    });

    test('subgraph should return undefined when no settings', () => {
      // Check the expected result with default empty settings
      expect(get(subgraph)).toBeUndefined();
    });

    test('subgraph should derive correctly when settings available', async () => {
      // Set test settings
      settings.set(mockConfig);

      // Set active orderbook
      activeOrderbookRef.set('orderbook1');

      // Check the expected result
      expect(get(subgraph)).toEqual({
        key: 'mainnet',
        url: 'https://api.thegraph.com/subgraphs/name/mainnet',
      });
    });

    test('accounts should return empty object when no settings', () => {
      // Check the expected result with default empty settings
      expect(get(accounts)).toEqual({});
    });

    test('activeAccounts should filter based on activeAccountsItems', () => {
      // Set test settings
      settings.set(mockConfig);

      // Set only one active account
      activeAccountsItems.set({
        name_one: 'address_one',
      });

      // Check the expected result
      expect(get(activeAccounts)).toEqual({
        name_one: {
          key: 'name_one',
          address: 'address_one',
        },
      });
    });

    test('activeAccounts should return empty when activeAccountsItems is empty', () => {
      // Set test settings
      settings.set(mockConfig);

      // Set no active accounts
      activeAccountsItems.set({});

      // Check the expected result
      expect(get(activeAccounts)).toEqual({});
    });
  });

  describe('Settings Subscription Edge Cases', () => {
    beforeEach(() => {
      // Reset all store values
      settings.set(EMPTY_SETTINGS);
      activeAccountsItems.set({});
    });

    test('should handle account items with extra accounts not in settings', () => {
      // Set up test settings with 2 accounts
      settings.set(mockConfig);

      // Set up active accounts including one not in settings
      activeAccountsItems.set({
        name_one: 'address_one', // Will be kept
        name_two: 'address_two', // Will be kept
        name_three: 'address_three', // Should be removed by the subscription
      });

      // Trigger the subscription by updating settings
      settings.set({ ...mockConfig });

      // Check the expected result - name_three should be removed
      expect(get(activeAccountsItems)).toEqual({
        name_one: 'address_one',
        name_two: 'address_two',
      });
    });
  });
}
