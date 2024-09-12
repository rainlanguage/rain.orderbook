import type { Vault } from '$lib/typeshare/subgraphTypes';
import { invoke } from '@tauri-apps/api';
import { mockIPC } from '@tauri-apps/api/mocks';

export type VaultDetailArgs = {
  id: string;
  subgraphArgs: {
    url: string;
  };
};

export const vaultDetail = async (id: string, url: string | undefined) => {
  if (!url) {
    return undefined;
  }
  return await invoke<Vault>('vault_detail', {
    id,
    subgraphArgs: { url },
  } as VaultDetailArgs);
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the vault_detail command correctly', async () => {
    const mockData: Vault = {
      id: '1',
      vaultId: '1',
      owner: '0x123',
      token: {
        id: '1',
        address: '0x456',
        name: 'USDC',
        symbol: 'USDC',
        decimals: '6',
      },
      balance: '100000000000',
      ordersAsInput: [],
      ordersAsOutput: [],
      balanceChanges: [],
      orderbook: {
        id: '0x00',
      },
    };
    mockIPC((cmd) => {
      if (cmd === 'vault_detail') {
        return mockData;
      }
    });

    // check for a result with no URL
    expect(await vaultDetail('1', undefined)).toEqual(undefined);

    // check for a result with a URL
    expect(await vaultDetail('1', 'http://localhost:8000')).toEqual({ ...mockData });
  });
}
