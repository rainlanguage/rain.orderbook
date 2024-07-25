import type { Vault } from '$lib/typeshare/vaultsList';
// import type { Vault as VaultDetail } from '$lib/typeshare/vaultDetail';
import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from './constants';
import { mockIPC } from '@tauri-apps/api/mocks';

export type VaultsListArgs = {
  subgraphArgs: {
    url: string;
  };
  paginationArgs: {
    page: number;
    page_size: number;
  };
};

export const vaultList = async (
  url: string | undefined,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!url) {
    return [];
  }
  return await invoke<Vault[]>('vaults_list', {
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  } as VaultsListArgs);
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the vaults_list command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'vaults_list') {
        return [
          {
            id: '1',
            vault_id: '1',
            owner: '0x123',
            token: {
              id: '1',
              address: '0x456',
              name: 'USDC',
              symbol: 'USDC',
              decimals: '6',
            },
            balance: '100000000000',
            orders_as_input: [],
            orders_as_output: [],
          },
        ];
      }
    });

    // check for a result with no URL
    expect(await vaultList(undefined, 0)).toEqual([]);

    // check for a result with a URL
    expect(await vaultList('http://localhost:8000', 0)).toEqual([
      {
        id: '1',
        vault_id: '1',
        owner: '0x123',
        token: {
          id: '1',
          address: '0x456',
          name: 'USDC',
          symbol: 'USDC',
          decimals: '6',
        },
        balance: '100000000000',
        orders_as_input: [],
        orders_as_output: [],
      },
    ]);
  });
}
