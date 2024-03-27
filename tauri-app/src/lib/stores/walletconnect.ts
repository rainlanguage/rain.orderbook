
import { get, writable } from '@square/svelte-store';
import find from 'lodash/find';
import * as chains from 'viem/chains';
import { type NetworkConfigSource } from '$lib/typeshare/configString';
import { createWeb3Modal, defaultConfig } from '@web3modal/ethers5'
import { activeNetwork } from './settings';
import { reportErrorToSentry } from '$lib/services/sentry';

const WALLETCONNECT_PROJECT_ID = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID;

const ethersConfig = defaultConfig({
  metadata: {
    name: "Raindex",
    description: "Raindex allows anyone to write, test, deploy and manage token trading strategies written in rainlang, on any EVM network.",
    url: "https://rainlang.xyz",
    icons: [
      "https://raw.githubusercontent.com/rainlanguage/dotrain/main/assets/rainlang-banner.svg",
      "https://raw.githubusercontent.com/WalletConnect/walletconnect-assets/master/Logo/Blue%20(Default)/Logo.svg",
    ]
  },
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
    } catch (e) {
      reportErrorToSentry(e);
      walletconnectModal.set(undefined);
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
        projectId: WALLETCONNECT_PROJECT_ID,
        enableOnramp: true,
        allWallets: "SHOW",
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

function getNetwork(network: NetworkConfigSource, chain?: chains.Chain) {
  return {
    chainId: network['chain-id'],
    name: chain?.name ?? `network with chain id: ${network['chain-id']}`,
    currency: chain?.nativeCurrency.symbol ?? network.currency ?? "eth",
    explorerUrl: chain?.blockExplorers?.default.url ?? "",
    rpcUrl: network.rpc
  }
}