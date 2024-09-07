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
  type OrderbookRef,
  type OrderbookConfigSource,
} from '$lib/typeshare/config';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { toasts } from './toasts';
import { pickBy } from 'lodash';
import { parseConfigSource } from '$lib/services/config';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

// general
export const settingsText = cachedWritableStore<string>(
  'settings',
  '',
  (s) => s,
  (s) => s,
);
export const settingsFile = textFileStore(
  'Orderbook Settings Yaml',
  ['yml', 'yaml'],
  get(settingsText),
);
export const settings = asyncDerived(
  settingsText,
  async ($settingsText): Promise<ConfigSource | undefined> => {
    try {
      const config: ConfigSource = await parseConfigSource($settingsText);
      return config;
    } catch (e) {
      reportErrorToSentry(e, SentrySeverityLevel.Info);
      toasts.error(e as string);
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
    await settings.load();
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
        ) as Record<OrderbookRef, OrderbookConfigSource>)
      : ({} as Record<OrderbookRef, OrderbookConfigSource>),
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

// watchlist
export const watchlist = derived(settings, ($settings) => $settings?.watchlist ?? {});

export const activeWatchlistItems = cachedWritableStore<Record<string, string>>(
  'settings.activeWatchlistItems',
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

export const activeWatchlist = derived(
  [watchlist, activeWatchlistItems],
  ([$watchlist, $activeWatchlistItems]) =>
    Object.keys($activeWatchlistItems).length === 0
      ? {}
      : Object.fromEntries(
          Object.entries($watchlist).filter(([key]) => key in $activeWatchlistItems),
        ),
);

// When networks / orderbooks settings updated, reset active network / orderbook
settings.subscribe(async () => {
  const $settings = await settings.load();
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
  await settings.load();
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
