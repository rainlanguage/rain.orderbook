import { toasts } from './toasts';
import { colorTheme } from './darkMode';
import { activeNetwork } from './settings';
import { get, writable } from '@square/svelte-store';
import Provider from '@walletconnect/ethereum-provider';
import { WalletConnectModal } from '@walletconnect/modal';
import { reportErrorToSentry } from '$lib/services/sentry';

const WALLETCONNECT_PROJECT_ID = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID;
const metadata = {
  name: "Raindex",
  description: "Raindex allows anyone to write, test, deploy and manage token trading strategies written in rainlang, on any EVM network.",
  url: "https://rainlang.xyz",
  icons: [
    "https://raw.githubusercontent.com/rainlanguage/rain.brand/main/Raindex%20logo.svg",
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
    optionalEvents: [
      "chainChanged",
      "accountsChanged",
      "connect",
      "disconnect",
    ],
    showQrModal: true,
    qrModalOptions: {
      themeMode: get(colorTheme),
      enableExplorer: false
    },
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

  walletconnectProvider = provider;

  // disconnect if last session is still active
  if (provider.accounts.length) {
    await walletconnectDisconnect();
  }
}).catch(e => {
  toasts.error("Cuuld not instantiate Walletconnect modal")
  reportErrorToSentry(e);
});

export async function walletconnectConnect() {
  if (walletconnectProvider?.accounts?.length) {
    await walletconnectDisconnect();
  } else {
    walletconnectIsConnecting.set(true);
    const network = get(activeNetwork);
    if (network) {
      const rpcMap: Record<string, string> = {};
      rpcMap[network['chain-id']] = network.rpc;
      try {
        await walletconnectProvider?.connect({
          chains: [network['chain-id']],
          rpcMap
        })
      } catch {
        toasts.error("Connection cancelled by user")
      }
    } else {
      toasts.error("Cannot find active network")
    }
    walletconnectIsConnecting.set(false);
  }
}

export async function walletconnectDisconnect() {
  walletconnectIsDisconnecting.set(true);
  try {
    await walletconnectProvider?.disconnect();
  } catch (e) {
    reportErrorToSentry(e);
  }
  walletconnectIsDisconnecting.set(false);
  walletconnectAccount.set(undefined);
}

// subscribe to networks and disconnect on network changes
activeNetwork.subscribe(async () => await walletconnectDisconnect());

// set theme when changed by user
colorTheme.subscribe(v => (walletconnectProvider?.modal as WalletConnectModal)?.setTheme({ themeMode: v }))