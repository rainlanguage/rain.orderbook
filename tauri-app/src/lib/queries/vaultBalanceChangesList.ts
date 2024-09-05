import type { VaultBalanceChangeUnwrapped } from '$lib/typeshare/subgraphTypes';
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
  return await invoke<VaultBalanceChangeUnwrapped[]>('vault_balance_changes_list', {
    id,
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  } as VaultBalanceChangesListArgs);
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the vault_balance_changes_list command correctly', async () => {
    const mockVaultBalanceChanges: VaultBalanceChangeUnwrapped[] = [
      {
        __typename: 'Withdrawal',
        amount: '1000',
        old_vault_balance: '5000',
        new_vault_balance: '4000',
        vault: {
          id: 'vault1',
          token: {
            id: 'token1',
            address: '0xTokenAddress1',
            name: 'Token1',
            symbol: 'TKN1',
            decimals: '18',
          },
        },
        timestamp: '1625247600',
        transaction: {
          id: 'tx1',
          from: '0xUser1',
          block_number: '0',
          timestamp: '0',
        },
        orderbook: {
          id: '0x00',
        },
      },
      {
        __typename: 'TradeVaultBalanceChange',
        amount: '1500',
        old_vault_balance: '4000',
        new_vault_balance: '2500',
        vault: {
          id: 'vault2',
          token: {
            id: 'token2',
            address: '0xTokenAddress2',
            name: 'Token2',
            symbol: 'TKN2',
            decimals: '18',
          },
        },
        timestamp: '1625347600',
        transaction: {
          id: 'tx2',
          from: '0xUser2',
          block_number: '0',
          timestamp: '0',
        },
        orderbook: {
          id: '0x00',
        },
      },
      {
        __typename: 'Deposit',
        amount: '2000',
        old_vault_balance: '2500',
        new_vault_balance: '4500',
        vault: {
          id: 'vault3',
          token: {
            id: 'token3',
            address: '0xTokenAddress3',
            name: 'Token3',
            symbol: 'TKN3',
            decimals: '18',
          },
        },
        timestamp: '1625447600',
        transaction: {
          id: 'tx3',
          from: '0xUser3',
          block_number: '0',
          timestamp: '0',
        },
        orderbook: {
          id: '0x00',
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
