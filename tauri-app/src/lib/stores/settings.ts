import { asyncDerived, derived, get } from '@square/svelte-store';
import { cachedWritableInt, cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';
import  find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import { invoke } from '@tauri-apps/api';
import { type Config } from '$lib/typeshare/config';
import { getBlockNumberFromRpc } from '$lib/services/chain';

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
};

// dotrain text store
export const dotrainFile = textFileStore('Rain', ['rain']);

// general
export const settingsText = cachedWritableStore<string>('settings', "", (s) => s, (s) => s);
export const settingsFile = textFileStore('Orderbook Settings Yaml', ['yml', 'yaml'], get(settingsText));
export const settings = asyncDerived([settingsText, dotrainFile], async ([$settingsText, $dotrainFile]): Promise<Config> => {
  const text = $dotrainFile?.text !== undefined ? $dotrainFile.text : "";
  try {
    const config: Config = await invoke("get_config", {dotrain: text, settingText: $settingsText});
    return config;
  } catch(e) {
    // eslint-disable-next-line no-console
    console.log(e);
    return emptyConfig;
  }
}, { initial: emptyConfig });

// networks
export const networks = derived(settings, ($settingsData) => Object.entries($settingsData?.networks));
export const activeNetworkIndex = cachedWritableInt("settings.activeNetworkIndex", 0);
export const activeNetwork = derived([networks, activeNetworkIndex], ([$networks, $activeNetworkIndex]) => $networks?.[$activeNetworkIndex]);
export const rpcUrl = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.[1].rpc);
export const chainId = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.[1].chain_id);
export const activeChain = derived(chainId, ($activeChainId) => find(Object.values(chains), (c) => c.id === $activeChainId));
export const activeChainHasBlockExplorer = derived(activeChain, ($activeChain) => {
  return $activeChain && $activeChain?.blockExplorers?.default !== undefined;
});
export const activeChainLatestBlockNumber = derived(activeNetwork, ($activeNetwork) => getBlockNumberFromRpc($activeNetwork?.[1].rpc));

// orderbook
export const orderbooks = derived([settings, activeNetwork], ([$settingsData, $activeNetwork]) => Object.entries($settingsData.orderbooks).filter(v =>
  // filter orderbooks based on active netowkr
  v[1].network.rpc === $activeNetwork?.[1].rpc
  && v[1].network.chain_id === $activeNetwork?.[1].chain_id
  && v[1].network.label === $activeNetwork?.[1].label
  && v[1].network.network_id === $activeNetwork?.[1].network_id
  && v[1].network.currency === $activeNetwork?.[1].currency
));
export const activeOrderbookIndex = cachedWritableInt("settings.activeOrderbookIndex", 0);
export const activeOrderbook =  derived([orderbooks, activeOrderbookIndex], ([$orderbooks, $activeOrderbookIndex]) => $orderbooks?.[$activeOrderbookIndex]);
export const subgraphUrl = derived(activeOrderbook, ($activeOrderbookSettings) => $activeOrderbookSettings?.[1].subgraph);
export const orderbookAddress = derived(activeOrderbook, ($activeOrderbookSettings) => $activeOrderbookSettings?.[1].address);

export const hasRequiredSettings = derived([activeNetwork, activeOrderbook], ([$activeChainSettings, $activeOrderbookSettings]) => true || ($activeChainSettings !== undefined && $activeOrderbookSettings !== undefined));

// deployments
export const deployments = derived([settings, activeNetwork, activeOrderbook], ([$settingsData, $activeNetwork, $activeOrderbook]) => Object.entries($settingsData.deployments).filter(v => {
  return v[1].order.network.rpc === $activeNetwork?.[1].rpc
    && v[1].order.network.chain_id === $activeNetwork?.[1].chain_id
    && v[1].order.network.label === $activeNetwork?.[1].label
    && v[1].order.network.network_id === $activeNetwork?.[1].network_id
    && v[1].order.network.currency === $activeNetwork?.[1].currency
    && (
      v[1].order.orderbook !== undefined
        ? (
          v[1].order.orderbook.address === $activeOrderbook?.[1].address
          && v[1].order.orderbook.subgraph === $activeOrderbook?.[1].subgraph
          && v[1].order.orderbook.label === $activeOrderbook?.[1].label
        )
        : true
    )
})
);
export const activeDeploymentIndex = cachedWritableInt("settings.activeDeploymentIndex", 0);
export const activeDeployment = derived([deployments, activeDeploymentIndex], ([$deployments, $activeDeploymentIndex]) => $deployments?.[$activeDeploymentIndex]);

// // When networks data updated, reset active chain
// networks.subscribe((val) => {
//   if(val && val.length < get(activeNetworkIndex)) {
//     activeNetworkIndex.set(0);
//   }
// });

// // When active network updated, reset active orderbook and deployment
// activeNetwork.subscribe(async ()  => {
//   activeOrderbookIndex.set(0);
//   activeDeploymentIndex.set(0);
// });