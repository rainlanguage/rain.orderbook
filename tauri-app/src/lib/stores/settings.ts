/* eslint-disable no-console */
import { asyncDerived, derived, get, writable } from '@square/svelte-store';
import { cachedWritableInt, cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';
import  find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import { invoke } from '@tauri-apps/api';
import { type Config } from '$lib/typeshare/config';
import { getBlockNumberFromRpc } from '$lib/services/chain';
import { toasts } from './toasts';
import { http, disconnect, createConfig, getAccount, type Config as WagmiConfig } from '@wagmi/core';
import type { Transport } from 'viem';
import { walletConnect } from '@wagmi/connectors'

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
    const config: Config = await invoke("merge_parse_configs", {dotrain: text, settingText: $settingsText});
    return config;
  } catch(e) {
    toasts.error(e as string);
    return emptyConfig;
  }
}, { initial: emptyConfig });

// networks
export const networks = derived(settings, ($settingsData) => Object.entries($settingsData?.networks));
export const activeNetworkIndex = cachedWritableInt("settings.activeNetworkIndex", 0);
export const activeNetwork = derived([networks, activeNetworkIndex], ([$networks, $activeNetworkIndex]) => $networks?.[$activeNetworkIndex]);
export const rpcUrl = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.[1].rpc);
export const chainId = derived(activeNetwork, ($activeNetwork) => $activeNetwork?.[1]["chain-id"]);
export const activeChain = derived(chainId, ($activeChainId) => find(Object.values(chains), (c) => c.id === $activeChainId));
export const activeChainHasBlockExplorer = derived(activeChain, ($activeChain) => {
  return $activeChain && $activeChain?.blockExplorers?.default !== undefined;
});
export const activeChainLatestBlockNumber = derived(activeNetwork, ($activeNetwork) => getBlockNumberFromRpc($activeNetwork?.[1].rpc));

// orderbook
export const orderbooks = derived([settings, activeNetwork], ([$settingsData, $activeNetwork]) => Object.entries($settingsData.orderbooks).filter(v =>
  // filter orderbooks based on active netowkr
  v[1].network.rpc === $activeNetwork?.[1].rpc
  && v[1].network["chain-id"] === $activeNetwork?.[1]["chain-id"]
  && v[1].network.label === $activeNetwork?.[1].label
  && v[1].network["network-id"] === $activeNetwork?.[1]["network-id"]
  && v[1].network.currency === $activeNetwork?.[1].currency
));
export const activeOrderbookIndex = cachedWritableInt("settings.activeOrderbookIndex", 0);
export const activeOrderbook =  derived([orderbooks, activeOrderbookIndex], ([$orderbooks, $activeOrderbookIndex]) => $orderbooks?.[$activeOrderbookIndex]);
export const subgraphUrl = derived(activeOrderbook, ($activeOrderbookSettings) => $activeOrderbookSettings?.[1].subgraph);
export const orderbookAddress = derived(activeOrderbook, ($activeOrderbookSettings) => $activeOrderbookSettings?.[1].address);

export const hasRequiredSettings = derived([activeNetwork, activeOrderbook], ([$activeChainSettings, $activeOrderbookSettings]) => $activeChainSettings !== undefined && $activeOrderbookSettings !== undefined);

// deployments
export const deployments = derived([settings, activeNetwork, activeOrderbook], ([$settingsData, $activeNetwork, $activeOrderbook]) => Object.entries($settingsData.deployments).filter(v => {
  return v[1].order.network.rpc === $activeNetwork?.[1].rpc
    && v[1].order.network["chain-id"] === $activeNetwork?.[1]["chain-id"]
    && v[1].order.network.label === $activeNetwork?.[1].label
    && v[1].order.network["network-id"] === $activeNetwork?.[1]["network-id"]
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

// wagmi config
const projectId = "634cfe0b2781e2ac78219ca4cb23c13f"
const metadata = {
  name: 'rain-ob',
  description: "some desc",
  url: 'https://rainlang.xyz', // origin must match your domain & subdomain
  icons: ['https://avatars.githubusercontent.com/u/37784886']
}
// const defaultWagmiConfig = createConfig({
//   chains: [chains.mainnet],
//   transports: {
//     [chains.mainnet.id]: http(chains.mainnet.rpcUrls.default.http[0])
//   },
//   connectors: [
//     walletConnect({ projectId, metadata, showQrModal: false }),
//   ],
// }) as WagmiConfig<[chains.Chain, ...chains.Chain[]], Record<number, Transport>>;

// export const wagmiConfig = derived([networks], ([$networks]) => {
//   if ($networks.length === 0) {
//     return;
//   }
//   // eslint-disable-next-line no-console
//   console.log("uuuuu1", $networks);
//   const viemchains: chains.Chain[] = [];
//   const transports: Record<number, Transport> = {};
//   for (const [, network] of $networks) {
//     const result = find(Object.values(chains), (c) => c.id === network["chain-id"]);
//     if (result) {
//       viemchains.push(result);
//       transports[result.id] = http(network.rpc);
//     }
//   }
//   // eslint-disable-next-line no-console
//   console.log("uuuuu2");
//   const config = createConfig({
//     chains: viemchains as [chains.Chain, ...chains.Chain[]],
//     transports,
//     connectors: [
//       walletConnect({ projectId, metadata, showQrModal: false }),
//     ],
//   });
//   // eslint-disable-next-line no-console
//   console.log("uuuuu3");
//   // const x = reconnect(config);
//   // await switchChain(config, { chainId: $activeNetwork[1]["chain-id"] });
//   return config;
// });

// // switch the chain for the wagmi config whenever active network changes
// // activeNetwork.subscribe(async (v) => {
// //   if (v) {
// //     const x = await switchChain(get(wagmiConfig), { chainId: v[1]["chain-id"] });
// //     // eslint-disable-next-line no-console
// //     console.log("wwwwww ", x);
// //   }
// // })

export const wagmiConfig = writable<WagmiConfig>();

// subscribe to networks and instantiate wagmi config store from it
activeNetwork.subscribe(async (network) => {
  const config = get(wagmiConfig);
  if (config && getAccount(config)?.isConnected) {
    try {
      await disconnect(config)
    } catch(e) {
      toasts.error(e as string)
    }
  }
  if (network) {
    const chain = find(Object.values(chains), (c) => c.id === network[1]["chain-id"]);
    if (chain) {
      wagmiConfig.set(
        createConfig({
          chains: [chain],
          transports: {
            [chain.id]: http(network[1].rpc),
          } as Record<number, Transport>,
          connectors: [
            walletConnect({ projectId, metadata, showQrModal: false }),
          ],
        })
      )
    }
    else {
      toasts.error("unsupported chain")
    }
  }
})
