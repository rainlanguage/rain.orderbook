import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from '@rainlanguage/ui-components';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { VaultWithSubgraphName } from '$lib/typeshare/subgraphTypes';

export type VaultsListArgs = {
  multiSubgraphArgs: {
    url: string;
    name: string;
  }[];
  filterArgs: {
    owners: string[];
    hideZeroBalance: boolean;
  };
  paginationArgs: {
    page: number;
    pageSize: number;
  };
};

export const vaultList = async (
  activeSubgraphs: Record<string, string>,
  owners: string[] = [],
  hideZeroBalance: boolean = true,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!Object.keys(activeSubgraphs).length) {
    return [];
  }
  return await invoke<VaultWithSubgraphName[]>('vaults_list', {
    multiSubgraphArgs: Object.entries(activeSubgraphs).map(([name, url]) => ({
      name,
      url,
    })),
    filterArgs: {
      owners,
      hideZeroBalance,
    },
    paginationArgs: { page: pageParam + 1, pageSize },
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
    expect(await vaultList({}, [], true, 0)).toEqual([]);

    // check for a result with a URL
    expect(await vaultList({ default: 'http://localhost:8000' }, [], true, 0)).toEqual([
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

    // check with hideZeroBalance set to false
    expect(await vaultList({ default: 'http://localhost:8000' }, [], false, 0)).toEqual([
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
