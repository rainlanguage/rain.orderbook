import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import WalletProvider from '../lib/providers/wallet/WalletProvider.svelte';
import { readable } from 'svelte/store';
import type { Account } from '$lib/types/account';

vi.mock('../lib/providers/wallet/context', () => ({
  setAccountContext: vi.fn()
}));

import { setAccountContext } from '../lib/providers/wallet/context';

describe('WalletProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should call setAccountContext with the account prop', () => {
    const mockAccount = readable('0x123') as Account;

    render(WalletProvider, {
      props: {
        account: mockAccount
      }
    });

    expect(setAccountContext).toHaveBeenCalledWith(mockAccount);
  });
});