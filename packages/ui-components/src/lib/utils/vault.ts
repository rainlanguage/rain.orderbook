import type { RaindexVault } from '@rainlanguage/orderbook';
import { formatUnits } from 'viem';

export const vaultBalanceDisplay = (vault: RaindexVault) => {
	return formatUnits(BigInt(vault.balance), Number(vault.token?.decimals || 0));
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
				address: '0x00'
			},
			vaultId: '1',
			owner: '0x00',
			ordersAsInput: [],
			ordersAsOutput: [],
			balanceChanges: [],
			orderbook: {
				id: '0x00'
			}
		};
		expect(vaultBalanceDisplay(vault as unknown as RaindexVault)).toEqual('1');

		vault.token.decimals = '6';
		expect(vaultBalanceDisplay(vault as unknown as RaindexVault)).toEqual('1000000000000');
	});
}
