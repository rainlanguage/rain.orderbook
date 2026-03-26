import { render, waitFor } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChart from '../lib/components/charts/VaultBalanceChart.svelte';
import type { RaindexVault } from '@rainlanguage/orderbook';
import { get, writable } from 'svelte/store';
import type { ComponentProps } from 'svelte';
import { props } from '../lib/__mocks__/MockComponent';

type VaultBalanceChartProps = ComponentProps<VaultBalanceChart>;
type MockChartProps = {
	priceSymbol?: string;
	query: {
		subscribe: (run: (value: unknown) => void) => () => void;
	};
};

vi.mock('../lib/components/charts/TanstackLightweightChartLine.svelte', async () => {
	const MockLightweightChart = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockLightweightChart };
});

test('calls getBalanceChanges with correct arguments', async () => {
	const queryClient = new QueryClient();
	const mockVault: RaindexVault = {
		id: 'vault1',
		token: {
			symbol: 'TKN'
		},
		getBalanceChanges: vi.fn().mockResolvedValue({
			value: [],
			error: null
		})
	} as unknown as RaindexVault;
	let unsubscribe = () => {};
	render(VaultBalanceChart, {
		props: {
			vault: mockVault,
			lightweightChartsTheme: writable({})
		} as VaultBalanceChartProps,
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		const mockProps = get(props) as MockChartProps;
		expect(mockProps.priceSymbol).toBe('TKN');
		unsubscribe = mockProps.query.subscribe(() => undefined);
	});

	await waitFor(() => {
		expect(mockVault.getBalanceChanges).toHaveBeenCalledWith(1);
	});
	unsubscribe();
});
