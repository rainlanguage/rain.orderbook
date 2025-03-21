import { getContext } from 'svelte';
import type { Readable } from 'svelte/store';
import { ACCOUNT_KEY, USE_ACCOUNT_KEY, type UseAccount } from './WalletProvider.svelte';

/**
 * Retrieves the useAccount function from Svelte's context
 */
export const getUseAccountContext = (): UseAccount | undefined => {
  const useAccountFn = getContext<UseAccount>(USE_ACCOUNT_KEY);
  return useAccountFn;
};

/**
 * Retrieves the account store directly from Svelte's context
 */
export const getAccountContext = (): Readable<string | null> => {
  const account = getContext<Readable<string | null>>(ACCOUNT_KEY);
  if (!account) {
    throw new Error(
      'No account was found in Svelte context. Did you forget to wrap your component with WalletProvider?'
    );
  }
  return account;
};