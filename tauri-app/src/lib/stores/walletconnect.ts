import { toasts } from './toasts';
import * as chains from 'viem/chains';
import { colorTheme } from './darkMode';
import { activeNetwork } from './settings';
import { get, writable } from '@square/svelte-store';
import Provider from '@walletconnect/ethereum-provider';
import { WalletConnectModal } from '@walletconnect/modal';
import { reportErrorToSentry } from '$lib/services/sentry';
import find from 'lodash/find';

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
export const walletconnectIsDisconnecting = writable<boolean>(false);
export const walletconnectIsConnecting = writable<boolean>(false);
export let walletconnectProvider: Provider | undefined;

Provider.init(
  {
    metadata,
    projectId: WALLETCONNECT_PROJECT_ID,
    optionalChains: [1],
    // Object.values(chains).map(v => v.id) as [number, ...number[]],
    optionalEvents: [
      "chainChanged",
      "accountsChanged",
      "connect",
      "disconnect",
      // "display_uri",
      // "session_event"
    ],
    showQrModal: true,
    qrModalOptions: { themeMode: get(colorTheme) },
  }
).then(async provider => {
  provider.on("connect", () => {
    walletconnectAccount.set(provider?.accounts?.[0] ?? undefined);
  });
  provider.on("disconnect", () => {
    walletconnectAccount.set(undefined);
  });
  provider.on("accountsChanged", (accounts) => {
    walletconnectAccount.set(accounts?.[0] ?? undefined);
  });
  provider.on("chainChanged", async (chainid) => {
    console.log("jjjjj");
    console.log(chainid);
    const network = get(activeNetwork);
    if (!network || network['chain-id'] !== Number(chainid)) {
      await walletconnectDisconnect();
    }
  });
  walletconnectProvider = provider;

  // disconnect if last session is still active
  if (provider.accounts.length) {
    await walletconnectDisconnect();
  }
}).catch(e => {
  toasts.error("could not instantiate walletconnect service")
  reportErrorToSentry(e);
});

export async function walletconnectConnect() {
  console.log(walletconnectProvider?.accounts);
  if (walletconnectProvider?.accounts?.length) {
    await walletconnectDisconnect();
    // indexedDB.deleteDatabase("WALLET_CONNECT_V2_INDEXED_DB")
  } else {
    walletconnectIsConnecting.set(true);
    const network = get(activeNetwork);
    if (network) {
      const rpcMap: Record<string, string>  = {};
      rpcMap[network['chain-id']] = network.rpc;
      try {
        await walletconnectProvider?.connect({
          chains: [network['chain-id']],
          rpcMap
        })
      } catch {
        toasts.error("canceled")
      }
    } else {
      toasts.error("Cannot find active network")
    }
    walletconnectIsConnecting.set(false);
  }
}

export async function walletconnectDisconnect() {
  console.log("disss")
  walletconnectAccount.set(undefined);
  walletconnectIsDisconnecting.set(true);
  try {
    await walletconnectProvider?.disconnect();
  } catch(e) {
    console.log(e);
    reportErrorToSentry(e);
  }
  walletconnectIsDisconnecting.set(false);
}

// subscribe to networks and instantiate wagmi config store from it
activeNetwork.subscribe(async network => {
  walletconnectIsConnecting.set(true);
  console.log(walletconnectProvider?.accounts);
  if (network && walletconnectProvider?.accounts?.length) {
    try {
      console.log("ccc", walletconnectProvider?.chainId);
      const x = await walletconnectProvider?.request({
        method: "wallet_switchEthereumChain",
        params: [{ chainId: network['chain-id'].toString(16) }],
      });
      console.log(x);
      console.log("done")
    } catch(e) {
      const errMsg = getErrorMsg(e);
      console.log("eeeeeeeeeeee", errMsg);
      if (errMsg.includes("Unrecognized chain ID")) {
        const chainDetails = getChainDetails(network['chain-id']);
        if (chainDetails) {
          try {
            await walletconnectProvider?.request({
              method: "wallet_addEthereumChain",
              params: [chainDetails],
            });
            // try {
            //   await walletconnectProvider?.request({
            //     method: "wallet_switchEthereumChain",
            //     params: [{ chainId: network['chain-id'].toString(16) }],
            //   });
            // } catch (swicthChainErr) {
            //   toasts.error(`could not siwtch network, reason: ${getErrorMsg(swicthChainErr)}`);
            // }
          } catch(addChainErr) {
            console.log(addChainErr)
            await walletconnectDisconnect();
            toasts.error(`could not add network, reason: ${getErrorMsg(addChainErr)}`);
          }
        }
      } else {
        console.log(e);
        toasts.error(`could not siwtch network, reason: ${errMsg}`);
        await walletconnectDisconnect();
      }
    }
  } else if (walletconnectProvider?.accounts?.length) {
    await walletconnectDisconnect();
  }
  walletconnectIsConnecting.set(false);
});

// set theme when changed by user
colorTheme.subscribe(v => (walletconnectProvider?.modal as WalletConnectModal)?.setTheme({themeMode: v}))

type ChainDetails = {
  chainId: string;
  blockExplorerUrls: [string];
  chainName: string;
  nativeCurrency: {
    name: string;
    symbol: string;
    decimals: number;
  };
  rpcUrls: [string];
};

function getChainDetails(chainid: number): ChainDetails | undefined {
  const chain = find(Object.values(chains), (c) => c.id === chainid);
  if (chain && chain.blockExplorers?.default.url) {
    return {
      chainId: "0x" + chainid.toString(16),
      blockExplorerUrls: [chain.blockExplorers.default.url],
      chainName: chain.name,
      nativeCurrency: chain.nativeCurrency,
      rpcUrls: chain.rpcUrls.default.http as [string]
    }
  }
  return undefined;
}

function getErrorMsg(e: unknown): string {
  if (e instanceof Error) {
    try {
      const parsedMsg = JSON.parse((e as Error).message);
      if (typeof parsedMsg.message === "string") return parsedMsg.message;
      else return (e as Error).message;
    } catch(err) {
      return (e as Error).message;
    }
  } else if (typeof e === "object" && e && "message" in e) {
    try {
      const parsedMsg = JSON.parse(e.message as string);
      if (typeof parsedMsg.message === "string") return parsedMsg.message;
      else return e.message as string;
    } catch(err) {
      return e.message as string;
    }
  }
  else {
    return `${e}`;
  }
}