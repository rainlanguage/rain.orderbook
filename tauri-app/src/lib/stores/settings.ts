import { asyncDerived, derived, get } from '@square/svelte-store';
import { cachedWritableStore, cachedWritableStringOptional } from '@rainlanguage/ui-components';
import find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import {
  parseYaml,
  type NewConfig,
  type OrderbookCfg,
  type SubgraphCfg,
} from '@rainlanguage/orderbook';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { pickBy } from 'lodash';

export const EMPTY_SETTINGS: NewConfig = {
  orderbook: {
    version: '1',
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
export const settings = cachedWritableStore<NewConfig>(
  'settings',
  EMPTY_SETTINGS,
  (value) => JSON.stringify(value),
  () => {
    try {
      const text = get(settingsText);
      const res = parseYaml([text]);
      if (res.error) {
        throw new Error(res.error.readableMsg);
      }
      return res.value;
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
export const activeNetwork = asyncDerived(
  [settings, activeNetworkRef],
  async ([$settings, $activeNetworkRef]) => {
    return $activeNetworkRef !== undefined && $settings?.orderbook.networks !== undefined
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
    $settings?.orderbook.orderbooks !== undefined && $activeOrderbookRef !== undefined
      ? $settings.orderbook.orderbooks[$activeOrderbookRef]
      : undefined,
);
export const subgraph = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
  Object.keys($settings.orderbook.subgraphs).length > 0 && $activeOrderbook?.subgraph !== undefined
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
  $settings?.orderbook.subgraphs !== undefined ? Object.entries($settings.orderbook.subgraphs) : [],
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
    Object.keys($settings.orderbook.networks).length === 0 ||
    $activeNetworkRef === undefined ||
    (Object.keys($settings.orderbook.networks).length > 0 &&
      $activeNetworkRef !== undefined &&
      !Object.keys($settings.orderbook.networks).includes($activeNetworkRef))
  ) {
    resetActiveNetworkRef();
  }

  if (
    Object.keys($settings.orderbook.orderbooks).length === 0 ||
    $activeOrderbookRef === undefined ||
    (Object.keys($settings.orderbook.orderbooks).length > 0 &&
      $activeOrderbookRef !== undefined &&
      !Object.keys($settings.orderbook.orderbooks).includes($activeOrderbookRef))
  ) {
    resetActiveOrderbookRef();
  }

  // Reset active account items if accounts have changed
  if (Object.keys($settings.orderbook.accounts ?? {}).length === 0) {
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
  if (Object.keys($settings.orderbook.subgraphs).length === 0) {
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
  const $networks = get(settings).orderbook.networks;

  if (Object.keys($networks).length > 0) {
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
