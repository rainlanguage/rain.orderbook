import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import type { OrderPerformance } from '@rainlanguage/orderbook';
import { QueryClient } from '@tanstack/svelte-query';
import OrderApy from '../lib/components/tables/OrderAPY.svelte';
import { bigintStringToPercentage } from '../lib/utils/number';

vi.mock('@rainlanguage/orderbook', () => ({
	getOrderPerformance: vi.fn(() => Promise.resolve(mockOrderApy))
}));

const mockOrderApy: OrderPerformance = {
	orderId: '1',
	orderHash: '1',
	orderbook: '1',
	denominatedPerformance: {
		apy: '1200000000000000000',
		apyIsNeg: true,
		token: {
			id: 'output_token',
			address: 'output_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: '0'
		},
		netVol: '0',
		netVolIsNeg: false,
		startingCapital: '0'
	},
	startTime: 1,
	endTime: 2,
	inputsVaults: [],
	outputsVaults: []
};
test('renders table with correct data', async () => {
	const queryClient = new QueryClient();

	render(OrderApy, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { id: '1', subgraphUrl: 'https://example.com' }
	});

	await waitFor(async () => {
		// get apy row
		const rows = screen.getAllByTestId('apy-field');

		// checking
		const display =
			(mockOrderApy.denominatedPerformance!.apyIsNeg ? '-' : '') +
			bigintStringToPercentage(mockOrderApy.denominatedPerformance!.apy, 18, 5);

		expect(rows[0]).toHaveTextContent(display);
	});
});
