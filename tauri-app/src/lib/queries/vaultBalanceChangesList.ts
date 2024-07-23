import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';
import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from './constants';
import { mockIPC } from '@tauri-apps/api/mocks';

export type VaultBalanceChangesListArgs = {
  id: string;
  subgraphArgs: {
    url: string;
  };
  paginationArgs: {
    page: number;
    page_size: number;
  };
};

export const vaultBalanceChangesList = async (
  id: string,
  url: string | undefined,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!url) {
    return [];
  }
  return await invoke<VaultBalanceChange[]>('vault_balance_changes_list', {
    id,
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  } as VaultBalanceChangesListArgs);
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the vault_balance_changes_list command correctly', async () => {
    const mockVaultBalanceChanges: VaultBalanceChange[] = [
      {
        __typename: 'Withdrawal',
        data: {
          id: 'withdrawal1',
          __typename: 'Withdrawal',
          amount: '1000',
          old_vault_balance: '5000',
          new_vault_balance: '4000',
        },
      },
      {
        __typename: 'TradeVaultBalanceChange',
        data: {
          id: 'trade1',
          __typename: 'TradeVaultBalanceChange',
          amount: '1500',
          old_vault_balance: '4000',
          new_vault_balance: '2500',
        },
      },
      {
        __typename: 'Deposit',
        data: {
          id: 'deposit1',
          __typename: 'Deposit',
          amount: '2000',
          old_vault_balance: '2500',
          new_vault_balance: '4500',
        },
      },
    ];

    mockIPC((cmd) => {
      if (cmd === 'vault_balance_changes_list') {
        return mockVaultBalanceChanges;
      }
    });

    // check for a result with no URL
    expect(await vaultBalanceChangesList('1', undefined, 0)).toEqual([]);

    // check for a result with a URL
    expect(await vaultBalanceChangesList('1', 'http://localhost:8000', 0)).toEqual(
      mockVaultBalanceChanges,
    );
  });
}
