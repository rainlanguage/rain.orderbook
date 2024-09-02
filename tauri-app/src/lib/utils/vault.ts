import type { Vault } from '$lib/typeshare/vaultsList';
import { formatUnits } from 'viem';

export const vaultBalanceDisplay = (vault: Vault) => {
  return formatUnits(BigInt(vault.balance), +(vault.token?.decimals || 0));
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('formats the vault balance correctly', () => {
    const vault = {
      id: '1',
      balance: '1000000000000000000',
      token: {
        id: '1',
        decimals: '18',
        address: '0x00',
      },
      vault_id: '1',
      owner: '0x00',
      orders_as_input: [],
      orders_as_output: [],
      orderbook: {
        id: '0x00',
      },
    } as Vault;

    expect(vaultBalanceDisplay(vault)).toEqual('1');

    vault.token.decimals = '6';

    expect(vaultBalanceDisplay(vault)).toEqual('1000000000000');
  });
}
