import type { AccountCfg } from '@rainlanguage/orderbook';

export function getAccountsAsOptions(accounts: Record<string, AccountCfg>) {
	return Object.fromEntries(Object.entries(accounts).map(([key, value]) => [key, value.address]));
}

if (import.meta.vitest) {
	const { expect, it, describe } = import.meta.vitest;

	describe('getAccountsAsOptions', () => {
		it('should return the correct accounts as options', () => {
			const accounts = {
				account1: { address: '0x1234567890abcdef', key: 'account1' },
				account2: { address: '0xabcdef1234567890', key: 'account2' }
			};
			const result = getAccountsAsOptions(accounts);
			expect(result).toEqual({
				account1: '0x1234567890abcdef',
				account2: '0xabcdef1234567890'
			});
		});
	});
}
