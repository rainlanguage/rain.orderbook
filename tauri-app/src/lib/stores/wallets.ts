import { derived } from 'svelte/store';
import { walletconnectAccount } from '$lib/stores/walletconnect';
import { writable } from '@square/svelte-store';
import type { Hex } from 'viem';

export const ledgerWalletAddress = writable<Hex | null>(null);
export const ledgerWalletDerivationIndex = writable<number>(0);

export const walletAddressMatchesOrBlank = derived(
  [ledgerWalletAddress, walletconnectAccount],
  ([$ledgerWalletAddress, $walletconnectAccount]) => {
    return (otherAddress: string) => {
      const otherAddressLowercase = otherAddress.toLowerCase();
      return (
        $ledgerWalletAddress?.toLowerCase() === otherAddressLowercase ||
        $walletconnectAccount?.toLowerCase() === otherAddressLowercase
      );
    };
  },
);
