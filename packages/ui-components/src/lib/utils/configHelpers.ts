import type { AccountCfg, MultiSubgraphArgs, SubgraphCfg } from '@rainlanguage/orderbook';

export function getMultiSubgraphArgs(subgraphs: Record<string, SubgraphCfg>) {
	return Object.entries(subgraphs).map(([name, value]) => ({
		name,
		url: value.url
	})) satisfies MultiSubgraphArgs[];
}

export function getAccountsAsOptions(accounts: Record<string, AccountCfg>) {
	return Object.fromEntries(Object.entries(accounts).map(([key, value]) => [key, value.address]));
}

if (import.meta.vitest) {
	const { expect, it, describe } = import.meta.vitest;

	describe('getMultiSubgraphArgs', () => {
		it('should return the correct multi subgraph args', () => {
			const subgraphs = {
				subgraph1: { url: 'https://subgraph1.com', key: 'subgraph1' },
				subgraph2: { url: 'https://subgraph2.com', key: 'subgraph2' }
			};
			const result = getMultiSubgraphArgs(subgraphs);
			expect(result).toEqual([
				{ name: 'subgraph1', url: 'https://subgraph1.com' },
				{ name: 'subgraph2', url: 'https://subgraph2.com' }
			]);
		});
	});

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
