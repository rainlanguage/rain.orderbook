import { getUseAccountContext, getAccountContext } from './context';

/**
 * Hook to access wallet account information from context
 * Must be used within a component that is a child of WalletProvider
 */
export function useAccount() {
  // Try to get the useAccount function from context first
  const useAccountFn = getUseAccountContext();
  if (useAccountFn) {
    return useAccountFn();
  }
  
  // Fallback to direct context access if needed
  const account = getAccountContext();
  
  return {
    account
  };
}