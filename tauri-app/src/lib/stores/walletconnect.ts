import { toasts } from './toasts';
import { colorTheme } from './darkMode';
import { settings } from './settings';
import { get, writable } from '@square/svelte-store';
import Provider from '@walletconnect/ethereum-provider';
import { WalletConnectModal } from '@walletconnect/modal';
import { reportErrorToSentry } from '$lib/services/sentry';
import { hexToNumber, isHex } from 'viem';
import { getBalanceFromWallet } from '$lib/services/wallet'

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

export const walletconnectAccount = writable<string | undefined>(undefined);
export const walletconnectIsDisconnecting = writable<boolean>(false);
export const walletconnectIsConnecting = writable<boolean>(false);
export let walletconnectProvider: Provider | undefined;
export const walletConnectNetwork = writable<number>(0);
export const walletBalance = writable<string | undefined>(undefined);

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
    const $settings = get(settings);
    provider.on('connect', async () => {
      const account = provider?.accounts?.[0];
      walletconnectAccount.set(account ?? undefined);
      if (account && $settings) {
        try {
          const balance = await getBalanceFromWallet(account);
          // Assuming balance is a BigInt or number, directly set it
          console.log('WC Connect balance:', balance);
          walletBalance.set(balance);
        } catch (error) {
          console.error('Error fetching balance:', error);
          reportErrorToSentry(error);
        }
      }
    });
    provider.on('disconnect', () => {
      walletconnectAccount.set(undefined);
    });
    provider.on('accountsChanged', async (accounts) => {
      const account = accounts?.[0];
      walletconnectAccount.set(account ?? undefined);
      if (account && $settings) {
        try {
          const balance = await getBalanceFromWallet(account);
          // Assuming balance is a BigInt or number, directly set it
          console.log('WC Change balance:', balance);
          walletBalance.set(balance);
        } catch (error) {
          console.error('Error fetching balance:', error);
          reportErrorToSentry(error);
        }
      }
    });
    provider.on('chainChanged', async (chain) => {
      const chainId = isHex(chain) ? hexToNumber(chain) : parseInt(chain);
      walletConnectNetwork.set(chainId);
      const account = provider?.accounts?.[0];
      if (account && $settings) {
        try {
          const balance = await getBalanceFromWallet(account);
          // Assuming balance is a BigInt or number, directly set it
          console.log('WC chain change balance:', balance);
          walletBalance.set(balance);
        } catch (error) {
          console.error('Error fetching balance:', error);
          reportErrorToSentry(error);
        }
      }
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

export async function walletconnectConnect() {
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
          optionalChains: chains,
          rpcMap,
        });
      } catch (e) {
        if (e instanceof ErrorEvent) {
          toasts.error(e?.message);
        } else {
          ('Could not connect to WalletConnect provider.');
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
  walletconnectAccount.set(undefined);
}

// async function fetchBalance(account: string, chainId: number) {
//   const rpcMap: Record<string, string> = {};
//   const chains: number[] = [];

//   const $settings = get(settings);

//   if ($settings?.networks) {
//     for (const network of Object.values($settings.networks)) {
//       rpcMap[network['chain-id']] = network.rpc;
//       chains.push(network['chain-id']);
//     }

//     const rpcUrl = rpcMap[chainId.toString()];

//     if (!rpcUrl) {
//       // console.error('RPC URL not found for chain ID:', chainId);
//       return;
//     }

//     const provider = new ethers.providers.JsonRpcProvider(rpcUrl);
//     try {
//       return await provider.getBalance(account);
//       // console.log({provider})
//       // console.log(`Balance for account ${account} on chain ${chainId}: ${ethers.utils.formatEther(balance)} ${provider.network.chainId}`);
//     } catch (error) {
//       // console.error('Failed to fetch balance:', error);
//       // reportErrorToSentry(error);
//     }
//   }
// }

// set theme when changed by user
colorTheme.subscribe((v) =>
  (walletconnectProvider?.modal as WalletConnectModal)?.setTheme({ themeMode: v }),
);
