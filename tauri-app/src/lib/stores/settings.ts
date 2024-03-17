import { asyncDerived, derived, get, writable } from '@square/svelte-store';
import { cachedWritableStore, cachedWritableStringOptional } from '$lib/storesGeneric/cachedWritableStore';
import  find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import { type ConfigString, type NetworkString, type OrderbookRef, type OrderbookString } from '$lib/typeshare/configString';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { toasts } from './toasts';
import { pickBy } from 'lodash';
import { parseConfigString } from '$lib/services/config';
import { createWeb3Modal, defaultConfig } from '@web3modal/ethers5'

const emptyConfig = {
  deployments: {},
  networks: {},
  orderbooks: {},
  orders: {},
  subgraphs: {},
  tokens: {},
  deployers: {},
  scenarios: {},
  charts: {}
} as ConfigString;

// general
export const settingsText = cachedWritableStore<string>('settings', "", (s) => s, (s) => s);
export const settingsFile = textFileStore('Orderbook Settings Yaml', ['yml', 'yaml'], get(settingsText));
export const settings = asyncDerived(settingsText, async ($settingsText): Promise<ConfigString> => {
  try {
    const config: ConfigString = await parseConfigString($settingsText);
    return config;
  } catch(e) {
    toasts.error(e as string);
    return emptyConfig;
  }
}, { initial: emptyConfig });

// networks
export const activeNetworkRef = cachedWritableStringOptional("settings.activeNetworkRef");
export const activeNetwork = asyncDerived([settings, activeNetworkRef], async ([$settings, $activeNetworkRef]) => {
  await settings.load();
  return ($activeNetworkRef !== undefined && $settings.networks !== undefined) ? $settings.networks[$activeNetworkRef] : undefined;
});
export const rpcUrl = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.rpc);
export const chainId = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.['chain-id']);
export const activeChain = derived(chainId, ($activeChainId) => find(Object.values(chains), (c) => c.id === $activeChainId));
export const activeChainHasBlockExplorer = derived(activeChain, ($activeChain) => {
  return $activeChain && $activeChain?.blockExplorers?.default !== undefined;
});
export const activeChainLatestBlockNumber = derived(activeNetwork, ($activeNetwork) => $activeNetwork !== undefined ? getBlockNumberFromRpc($activeNetwork.rpc) : 0);

// orderbook
export const activeOrderbookRef = cachedWritableStringOptional("settings.activeOrderbookRef");
export const activeNetworkOrderbooks = derived([settings, activeNetworkRef], ([$settings, $activeNetworkRef]) => $settings?.orderbooks ? pickBy($settings.orderbooks, (orderbook) => orderbook.network === $activeNetworkRef) as Record<OrderbookRef, OrderbookString> : {} as Record<OrderbookRef, OrderbookString>);
export const activeOrderbook =  derived([settings, activeOrderbookRef], ([$settings, $activeOrderbookRef]) => ($settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined) ? $settings.orderbooks[$activeOrderbookRef] : undefined);
export const subgraphUrl = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) => ($settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined) ? $settings.subgraphs[$activeOrderbook.subgraph] : undefined);
export const orderbookAddress = derived(activeOrderbook, ($activeOrderbook) => $activeOrderbook?.address);

export const hasRequiredSettings = derived([activeNetworkRef, activeOrderbookRef], ([$activeNetworkRef, $activeOrderbookRef]) => $activeNetworkRef !== undefined && $activeOrderbookRef !== undefined);

// When networks / orderbooks settings updated, reset active network / orderbook
settings.subscribe(async ($settings) => {
  await settings.load();
  const $activeNetworkRef = get(activeNetworkRef);
  const $activeOrderbookRef = get(activeOrderbookRef);

  if(!$settings.networks || $activeNetworkRef === undefined || !Object.keys($settings.networks).includes($activeNetworkRef)) {
    resetActiveNetworkRef();
  }

  if(!$settings.orderbooks || $activeOrderbookRef === undefined || !Object.keys($settings.orderbooks).includes($activeOrderbookRef)) {
    resetActiveOrderbookRef();
  }
});

// When active network is updated to undefined, reset active orderbook
activeNetworkRef.subscribe(($activeNetworkRef)  => {
  if($activeNetworkRef === undefined) {
    resetActiveOrderbookRef();
  }
});

// When active network is updated to not include active orderbook, reset active orderbook
activeNetworkOrderbooks.subscribe(async ($activeNetworkOrderbooks) => {
  const $activeOrderbookRef = get(activeOrderbookRef);

  if($activeOrderbookRef !== undefined && !Object.keys($activeNetworkOrderbooks).includes($activeOrderbookRef)) {
    resetActiveOrderbookRef();
  }
});

// reset active orderbook to first available, otherwise undefined
function resetActiveOrderbookRef() {
  const $activeNetworkOrderbooks = get(activeNetworkOrderbooks);
  const $activeNetworkOrderbookRefs = Object.keys($activeNetworkOrderbooks);

  if($activeNetworkOrderbookRefs.length > 0) {
    activeOrderbookRef.set($activeNetworkOrderbookRefs[0]);
  } else {
    activeOrderbookRef.set(undefined);
  }
}


// reset active orderbook to first available, otherwise undefined
function resetActiveNetworkRef() {
  const $networks = get(settings).networks;

  if($networks !== undefined && Object.keys($networks).length > 0) {
    activeNetworkRef.set(Object.keys($networks)[0]);
  } else {
    activeNetworkRef.set(undefined);
  }
}

// @TODO set rain project id from env
const projectId = "634cfe0b2781e2ac78219ca4cb23c13f"

// @TODO set correct values for fields
const metadata = {
  name: "Rain Orderbook",
  description: "Rain Orderbook is an app to build and deploy token strategies (sell/buy, limit, treasury management and ...) that are written in RainLanguage, make charts for them, watch all the orderbooks orders and easily deposit, withdraw and manage orderbooks vaults.",
  url: "https://rainlang.xyz", // origin must match your domain & subdomain
  icons: [
    "https://raw.githubusercontent.com/rainlanguage/dotrain/main/assets/rainlang-banner.svg", // rain logo
    "https://avatars.githubusercontent.com/u/37784886", // walletconnect logo
  ]
}

const ethersConfig = defaultConfig({
  metadata,
  enableEIP6963: false,
  enableInjected: false,
  enableCoinbase: false,
});

export const walletconnectModal = writable<ReturnType<typeof createWeb3Modal> | undefined>();
export const walletconnectAccount = writable<string | undefined>(undefined);
export const walletconnectIsConnected = writable<boolean>(false);
let eventUnsubscribe: (() => void) | undefined;

// subscribe to networks and instantiate wagmi config store from it
activeNetwork.subscribe(async network => {
  if (eventUnsubscribe) eventUnsubscribe();
  walletconnectAccount.set(undefined);
  walletconnectIsConnected.set(false);
  const oldModal = get(walletconnectModal)
  if (oldModal !== undefined) {
    try {
      await oldModal.disconnect()
    } catch(e) {
      walletconnectModal.set(undefined)
      // eslint-disable-next-line no-console
      console.log(e)
    }
  }
  if (network === undefined) {
    walletconnectModal.set(undefined);
  }
  else {
    const chain = find(Object.values(chains), (c) => c.id === network["chain-id"]);
    walletconnectModal.set(
      createWeb3Modal({
        ethersConfig,
        chains: [getNetwork(network, chain)],
        projectId,
        enableAnalytics: true, // Optional - defaults to your Cloud configuration
        enableOnramp: true, // Optional - false as default
        allWallets: "SHOW",
        // includeWalletIds: [
        //   "e7c4d26541a7fd84dbdfa9922d3ad21e936e13a7a0e44385d44f006139e44d3b" // walletconnect
        // ],
      })
    )
    const modal = get(walletconnectModal);
    eventUnsubscribe = modal?.subscribeEvents(v => {
      if (v.data.event === "MODAL_CLOSE") {
        walletconnectAccount.set(modal.getAddress());
        walletconnectIsConnected.set(modal.getIsConnected());
      }
    })
  }
})

function getNetwork(network: NetworkString, chain?: chains.Chain) {
  return {
    chainId: network['chain-id'],
    name: chain?.name ?? `network with chain id: ${network['chain-id']}`,
    currency: chain?.nativeCurrency.symbol ?? network.currency ?? "eth",
    explorerUrl: chain?.blockExplorers?.default.url ?? "",
    rpcUrl: network.rpc
  }
}