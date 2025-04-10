import { render } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChart from '../lib/components/charts/VaultBalanceChart.svelte';
import type { SgVault } from '@rainlanguage/orderbook';
import { getVaultBalanceChanges } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';
import type { ComponentProps } from 'svelte';

type VaultBalanceChartProps = ComponentProps<VaultBalanceChart>;

vi.mock('@rainlanguage/orderbook', () => ({
	getVaultBalanceChanges: vi.fn()
}));

vi.mock('../lib/components/charts/TanstackLightweightChartLine.svelte', async () => {
	const MockLightweightChart = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockLightweightChart };
});

const mockVault: SgVault = {
	id: 'vault1',
	vaultId: 'vault1',
	token: {
		id: 'token1',
		address: '0xTokenAddress1',
		name: 'Token1',
		symbol: 'TKN1',
		decimals: '18'
	},
	owner: '0xOwnerAddress',
	ordersAsInput: [],
	ordersAsOutput: [],
	balanceChanges: [],
	balance: '1000000000000000000',
	orderbook: {
		id: '0x00'
	}
};

test('calls getVaultBalanceChanges with correct arguments', async () => {
	const queryClient = new QueryClient();

	render(VaultBalanceChart, {
		props: {
			vault: mockVault,
			subgraphUrl: 'https://example.com',
			lightweightChartsTheme: writable({}),
			id: 'vault1'
		} as VaultBalanceChartProps,
		context: new Map([['$$_queryClient', queryClient]])
	});

	expect(getVaultBalanceChanges).toHaveBeenCalledWith('https://example.com', 'vault1', {
		page: 1,
		pageSize: 1000
	});
});
