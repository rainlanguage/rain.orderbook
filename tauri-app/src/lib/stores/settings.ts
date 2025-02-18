import { asyncDerived, derived, get } from '@square/svelte-store';
import {
  cachedWritableStore,
  cachedWritableStringOptional,
} from '$lib/storesGeneric/cachedWritableStore';
import find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import {
  type ConfigSource,
  type CfgOrderbookRef,
  type OrderbookConfigSource,
} from '@rainlanguage/orderbook/js_api';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { pickBy } from 'lodash';

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
export const settings = cachedWritableStore<ConfigSource | undefined>(
  'settings',
  undefined,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      return JSON.parse(str) as ConfigSource;
    } catch {
      return undefined;
    }
  },
);
export const enableSentry = derived(settings, ($settings) =>
  $settings?.sentry !== undefined ? $settings.sentry : true,
);

// networks
export const activeNetworkRef = cachedWritableStringOptional('settings.activeNetworkRef');
export const activeNetwork = asyncDerived(
  [settings, activeNetworkRef],
  async ([$settings, $activeNetworkRef]) => {
    return $activeNetworkRef !== undefined && $settings?.networks !== undefined
      ? $settings.networks[$activeNetworkRef]
      : undefined;
  },
);
export const rpcUrl = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.rpc);
export const chainId = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.['chain-id']);
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
    $settings?.orderbooks
      ? (pickBy(
          $settings.orderbooks,
          (orderbook) => orderbook.network === $activeNetworkRef,
        ) as Record<CfgOrderbookRef, OrderbookConfigSource>)
      : ({} as Record<CfgOrderbookRef, OrderbookConfigSource>),
);
export const activeOrderbook = derived(
  [settings, activeOrderbookRef],
  ([$settings, $activeOrderbookRef]) =>
    $settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined
      ? $settings.orderbooks[$activeOrderbookRef]
      : undefined,
);
export const subgraphUrl = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
  $settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
    ? $settings.subgraphs[$activeOrderbook.subgraph]
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
export const accounts = derived(settings, ($settings) => $settings?.accounts ?? {});
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
export const subgraph = derived(settings, ($settings) =>
  $settings?.subgraphs !== undefined ? Object.entries($settings.subgraphs) : [],
);
export const activeSubgraphs = cachedWritableStore<Record<string, string>>(
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
    $settings?.networks === undefined ||
    $activeNetworkRef === undefined ||
    ($settings?.networks !== undefined &&
      $activeNetworkRef !== undefined &&
      !Object.keys($settings.networks).includes($activeNetworkRef))
  ) {
    resetActiveNetworkRef();
  }

  if (
    !$settings?.orderbooks === undefined ||
    $activeOrderbookRef === undefined ||
    ($settings?.orderbooks !== undefined &&
      $activeOrderbookRef !== undefined &&
      !Object.keys($settings.orderbooks).includes($activeOrderbookRef))
  ) {
    resetActiveOrderbookRef();
  }

  // Reset active account items if accounts have changed
  if ($settings?.accounts === undefined) {
    activeAccountsItems.set({});
  } else {
    const currentActiveAccounts = get(activeAccountsItems);
    const updatedActiveAccounts = Object.fromEntries(
      Object.entries($settings.accounts ?? {}).filter(([key, value]) => {
        if (key in currentActiveAccounts) {
          return currentActiveAccounts[key] === value;
        }
        return false;
      }),
    );
    activeAccountsItems.set(updatedActiveAccounts);
  }

  // Reset active subgraphs if subgraphs have changed
  if ($settings?.subgraphs === undefined) {
    activeSubgraphs.set({});
  } else {
    const currentActiveSubgraphs = get(activeSubgraphs);
    const updatedActiveSubgraphs = Object.fromEntries(
      Object.entries($settings.subgraphs).filter(([key, value]) => {
        if (key in currentActiveSubgraphs) {
          return currentActiveSubgraphs[key] === value;
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
  const $networks = get(settings)?.networks;

  if ($networks !== undefined && Object.keys($networks).length > 0) {
    activeNetworkRef.set(Object.keys($networks)[0]);
  } else {
    activeNetworkRef.set(undefined);
  }
}

export const activeOrderStatus = cachedWritableStore<boolean | undefined>(
  'settings.activeOrderStatus',
  undefined,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      const parsed = JSON.parse(str);
      return typeof parsed === 'boolean' ? parsed : undefined;
    } catch {
      return undefined;
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
