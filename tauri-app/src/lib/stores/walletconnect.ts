import { toasts } from './toasts';
import { colorTheme } from './darkMode';
import { settings } from './settings';
import { get, writable } from '@square/svelte-store';
import Provider from '@walletconnect/ethereum-provider';
import { WalletConnectModal } from '@walletconnect/modal';
import { reportErrorToSentry } from '$lib/services/sentry';
import { hexToNumber, isHex, type Hex } from 'viem';

const WALLETCONNECT_PROJECT_ID = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID;
const metadata = {
  name: 'Raindex',
  description:
    'Raindex allows anyone to write, test, deploy and manage token trading strategies written in rainlang, on any EVM network.',
  url: 'https://rainlang.xyz',
  icons: [
    'https://raw.githubusercontent.com/rainlanguage/rain.brand/main/Raindex%20logo.svg',
    'https://raw.githubusercontent.com/WalletConnect/walletconnect-assets/master/Logo/Blue%20(Default)/Logo.svg',
  ],
};

export const walletconnectAccount = writable<Hex | null>(null);
export const walletconnectIsDisconnecting = writable<boolean>(false);
export const walletconnectIsConnecting = writable<boolean>(false);
export let walletconnectProvider: Provider | undefined;
export const walletConnectNetwork = writable<number>(0);

Provider.init({
  metadata,
  projectId: WALLETCONNECT_PROJECT_ID,
  optionalChains: [1],
  optionalEvents: ['chainChanged', 'accountsChanged', 'connect', 'disconnect'],
  showQrModal: true,
  qrModalOptions: {
    themeMode: get(colorTheme),
    enableExplorer: false,
  },
})
  .then(async (provider) => {
    provider.on('connect', () => {
      walletconnectAccount.set((provider?.accounts?.[0] as Hex) ?? null);
    });
    provider.on('disconnect', () => {
      walletconnectAccount.set(null);
    });
    provider.on('accountsChanged', (accounts) => {
      walletconnectAccount.set((accounts?.[0] as Hex) ?? null);
    });
    provider.on('chainChanged', (chain) => {
      if (isHex(chain)) walletConnectNetwork.set(hexToNumber(chain));
      else walletConnectNetwork.set(parseInt(chain));
    });

    walletconnectProvider = provider;

    // disconnect if last session is still active
    if (provider.accounts.length) {
      await walletconnectDisconnect();
    }
  })
  .catch((e) => {
    toasts.error('Could not instantiate Walletconnect modal');
    reportErrorToSentry(e);
  });

export async function walletconnectConnect(priorityChainIds: number[]) {
  if (!walletconnectProvider?.accounts?.length) {
    walletconnectIsConnecting.set(true);
    const rpcMap: Record<string, string> = {};
    const chains: number[] = [];

    const $settings = get(settings);

    if ($settings?.networks) {
      for (const network of Object.values($settings.networks)) {
        rpcMap[network['chain-id']] = network.rpc;
        chains.push(network['chain-id']);
      }
      try {
        await walletconnectProvider?.connect({
          optionalChains: [...new Set([...priorityChainIds, ...chains])],
          rpcMap,
        });
      } catch (e) {
        if (e instanceof ErrorEvent) {
          toasts.error(e?.message);
        } else {
          return 'Could not connect to WalletConnect provider.';
        }
      }
    } else {
      toasts.error('No networks configured in settings.');
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
  walletconnectAccount.set(null);
}

// set theme when changed by user
colorTheme.subscribe((v) =>
  (walletconnectProvider?.modal as WalletConnectModal)?.setTheme({ themeMode: v }),
);
