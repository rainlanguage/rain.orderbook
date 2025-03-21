import { getContext } from 'svelte';
import type { Readable } from 'svelte/store';
import { ACCOUNT_KEY, USE_ACCOUNT_KEY, type UseAccount } from './WalletProvider.svelte';
import { readable } from 'svelte/store';
  import { useAccount } from './useAccount';


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

if (import.meta.vitest) {
  const { describe, it, expect, vi, beforeEach } = import.meta.vitest;
  
  vi.mock('svelte', () => ({
    getContext: vi.fn()
  }));
  
  describe('useAccount hook', () => {
    const mockGetContext = vi.mocked(getContext);
    
    beforeEach(() => {
      mockGetContext.mockReset();
    });

    it('should use the useAccount function from context when available', () => {
      const mockAccount = readable('0x123');
      const mockUseAccountFn = () => ({ account: mockAccount });
      
      mockGetContext.mockImplementation((key) => {
        if (key === USE_ACCOUNT_KEY) return mockUseAccountFn;
        return undefined;
      });
      
      const result = useAccount();
      
      expect(mockGetContext).toHaveBeenCalledWith(USE_ACCOUNT_KEY);
      expect(result).toEqual({ account: mockAccount });
    });

    it('should fall back to direct account context when useAccount function is not available', () => {
      const mockAccount = readable('0x456');
      
      mockGetContext.mockImplementation((key) => {
        if (key === USE_ACCOUNT_KEY) return undefined;
        if (key === ACCOUNT_KEY) return mockAccount;
        return undefined;
      });
      
      const result = useAccount();
      
      expect(mockGetContext).toHaveBeenCalledWith(USE_ACCOUNT_KEY);
      expect(mockGetContext).toHaveBeenCalledWith(ACCOUNT_KEY);
      expect(result).toEqual({ account: mockAccount });
    });
    
    it('should throw an error when account is not in context', () => {
      mockGetContext.mockReturnValue(undefined);
      
      expect(() => useAccount()).toThrow(
        'No account was found in Svelte context. Did you forget to wrap your component with WalletProvider?'
      );
    });
  });
}