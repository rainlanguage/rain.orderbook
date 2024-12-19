import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import type { VaultVolume } from '../lib/typeshare/subgraphTypes';
import { formatUnits } from 'viem';
import OrderVaultsVolTable from '../lib/components/tables/OrderVaultsVolTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

// Mock the getOrderVaultsVolume function
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getOrderVaultsVolume: vi.fn(() => Promise.resolve(mockVaultsVol))
}));

const mockVaultsVol: VaultVolume[] = [
	{
		id: '1',
		token: {
			id: 'output_token',
			address: 'output_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: '0'
		},
		totalIn: '1',
		totalOut: '2',
		totalVol: '3',
		netVol: '-1'
	},
	{
		id: '2',
		token: {
			id: 'output_token',
			address: 'output_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: '0'
		},
		totalIn: '2',
		totalOut: '5',
		totalVol: '7',
		netVol: '-3'
	}
];

test('renders table with correct data', async () => {
	const queryClient = new QueryClient();

	render(OrderVaultsVolTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { id: '1', subgraphUrl: 'https://example.com' }
	});

	await waitFor(async () => {
		// get total ins
		const rows = screen.getAllByTestId('total-in');

		// checking the total ins
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = formatUnits(
				BigInt(mockVaultsVol[i].totalIn),
				Number(mockVaultsVol[i].token.decimals)
			);
			expect(rows[i]).toHaveTextContent(display.toString());
		}
	});

	await waitFor(async () => {
		// get total outs
		const rows = screen.getAllByTestId('total-out');

		// checking the total outs
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = formatUnits(
				BigInt(mockVaultsVol[i].totalOut),
				Number(mockVaultsVol[i].token.decimals)
			);
			expect(rows[i]).toHaveTextContent(display.toString());
		}
	});

	await waitFor(async () => {
		// get net vols
		const rows = screen.getAllByTestId('net-vol');

		// checking the net vols
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = formatUnits(
				BigInt(mockVaultsVol[i].netVol),
				Number(mockVaultsVol[i].token.decimals)
			);
			expect(rows[i]).toHaveTextContent(display.toString());
		}
	});

	await waitFor(async () => {
		// get total vols
		const rows = screen.getAllByTestId('total-vol');

		// checking the total vols
		for (let i = 0; i < mockVaultsVol.length; i++) {
			const display = formatUnits(
				BigInt(mockVaultsVol[i].totalVol),
				Number(mockVaultsVol[i].token.decimals)
			);
			expect(rows[i]).toHaveTextContent(display.toString());
		}
	});
});
