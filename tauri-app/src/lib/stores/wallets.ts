import { derived } from 'svelte/store';
import { writable } from '@square/svelte-store';
import { useAccount } from '@rainlanguage/ui-components';

const { account } = useAccount();

export const ledgerWalletDerivationIndex = writable<number>(0);

export const walletAddressMatchesOrBlank = derived(
  [account],
  ([$account]) => {
    return (otherAddress: string) => {
      const otherAddressLowercase = otherAddress.toLowerCase();
      return (
        $account?.toLowerCase() === otherAddressLowercase
      );
    };
  },
);
