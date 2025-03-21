import { getContext } from 'svelte';
import type { Readable } from 'svelte/store';
import { SIGNER_ADDRESS_KEY, USE_ACCOUNT_KEY, type UseAccount } from '../providers/WalletProvider.svelte';

export function useAccount() {
  const useAccountFn: UseAccount = getContext(USE_ACCOUNT_KEY);

  if (useAccountFn) {
    return useAccountFn();
  }
  
  const account = getContext(SIGNER_ADDRESS_KEY) as Readable<string | null>;
  
  return {
    account
  };
}