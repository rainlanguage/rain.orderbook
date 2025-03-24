import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import WalletProvider, { ACCOUNT_KEY } from '../lib/providers/wallet/WalletProvider.svelte';
import { readable } from 'svelte/store';
import type { Hex } from 'viem';

// Mock the context module
vi.mock('./context', () => ({
  setAccountContext: vi.fn()
}));

// Import the mocked function after mocking
import { setAccountContext } from '../lib/providers/wallet/context';

describe('WalletProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should set account store in context via setAccountContext', () => {
    const mockAccount = readable('0x123' as unknown as Hex);

    render(WalletProvider, {
      props: {
        account: mockAccount
      }
    });

    expect(setAccountContext).toHaveBeenCalledWith(mockAccount);
  });

  it('should use default null account when no account prop provided', () => {
    render(WalletProvider);

    // Check that setAccountContext was called with a readable store containing null
    expect(setAccountContext).toHaveBeenCalled();
    
    const accountPassedToContext = vi.mocked(setAccountContext).mock.calls[0][0];
    
    // Subscribe to the store to get its value
    let value;
    const unsubscribe = accountPassedToContext.subscribe((val: Hex | null) => {
      value = val;
    });
    
    expect(value).toBeNull();
    unsubscribe();
  });


});