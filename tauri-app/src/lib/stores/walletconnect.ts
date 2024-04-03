
import { get, writable } from '@square/svelte-store';
import { activeNetwork } from './settings';
// import { reportErrorToSentry } from '$lib/services/sentry';
import Provider from '@walletconnect/ethereum-provider';
import { colorTheme } from './darkMode';
// import { WalletConnectModal, WalletConnectModalConfig } from '@walletconnect/modal';
import { toasts } from './toasts';
// import { EthersConstantsUtil, EthersHelpersUtil, EthersStoreUtil } from '@web3modal/scaffold-utils/ethers';
import * as chains from 'viem/chains';

const WALLETCONNECT_PROJECT_ID = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID;

const metadata = {
  name: "Raindex",
  description: "Raindex allows anyone to write, test, deploy and manage token trading strategies written in rainlang, on any EVM network.",
  url: "https://rainlang.xyz",
  icons: [
    "https://raw.githubusercontent.com/rainlanguage/dotrain/main/assets/rainlang-banner.svg",
    "https://raw.githubusercontent.com/WalletConnect/walletconnect-assets/master/Logo/Blue%20(Default)/Logo.svg",
  ]
};

export const walletconnectAccount = writable<string | undefined>(undefined);
export const walletconnectIsConnected = writable<boolean>(false);

export let walletconnectProvider: Provider | undefined;
(async() => {
  const optionalChains = Object.values(chains).map(v => v.id) as [number, ...number[]];
  walletconnectProvider = await Provider.init({
    metadata,
    projectId: WALLETCONNECT_PROJECT_ID,
    showQrModal: true,
    optionalChains,
    qrModalOptions: { themeMode: get(colorTheme) },
    optionalEvents: ["chainChanged", "accountsChanged", "connect", "disconnect", "display_uri", "session_event"],
  });
  walletconnectProvider.on("connect", () => {
    console.log(walletconnectProvider?.accounts);
    console.log(walletconnectProvider?.chainId);
    walletconnectAccount.set(walletconnectProvider?.accounts[0]);
    walletconnectIsConnected.set(true);
  });
  walletconnectProvider.on("disconnect", () => {
    console.log("disssssssssss");
    walletconnectAccount.set(undefined);
    walletconnectIsConnected.set(false);
  });
  walletconnectProvider.on("accountsChanged", (acc) => {
    console.log("yoyoyoyo");
    console.log(walletconnectProvider?.chainId);
    walletconnectAccount.set(acc[0]);
  });
  walletconnectProvider.on("chainChanged", async (chainid) => {
    console.log("nnnnnnnn");
    console.log(walletconnectProvider?.chainId);
    console.log(chainid);
    await walletconnectDisconnect();
  });
})();

export async function walletconnectConnect() {
  if (get(walletconnectIsConnected)) {
    await walletconnectDisconnect();
  } else {
    const network = get(activeNetwork);
    if (network) {
      const rpcMap: Record<string, string>  = {};
      rpcMap[network['chain-id']] = network.rpc;
      await walletconnectProvider?.connect({
        chains: [network['chain-id']],
        rpcMap
      })
    } else {
      toasts.error("Cannot find active network")
    }
  }
}

export async function walletconnectDisconnect() {
  try {
    await walletconnectProvider?.disconnect();
  } catch(e) {
    console.log(e)
  }
  walletconnectAccount.set(undefined);
  walletconnectIsConnected.set(false);
}

// subscribe to networks and instantiate wagmi config store from it
activeNetwork.subscribe(async () => await walletconnectDisconnect())