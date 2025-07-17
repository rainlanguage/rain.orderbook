import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import type { RaindexOrder, RaindexVaultVolume } from '@rainlanguage/orderbook';
import OrderVaultsVolTable from '../lib/components/tables/OrderVaultsVolTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

const mockVaultsVol = [
	{
		id: '1',
		token: {
			id: 'output_token',
			address: '0xoutput_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: BigInt('0')
		},
		details: {
			totalIn: '1',
			formattedTotalIn: '1',
			totalOut: '2',
			formattedTotalOut: '2',
			totalVol: '3',
			formattedTotalVol: '3',
			netVol: '1',
			formattedNetVol: '1'
		}
	},
	{
		id: '2',
		token: {
			id: 'output_token',
			address: '0xoutput_token',
			name: 'output_token',
			symbol: 'output_token',
			decimals: BigInt('0')
		},
		details: {
			totalIn: '2',
			formattedTotalIn: '2',
			totalOut: '5',
			formattedTotalOut: '5',
			totalVol: '7',
			formattedTotalVol: '7',
			netVol: '3',
			formattedNetVol: '3'
		}
	}
] as unknown as RaindexVaultVolume[];

// TODO: Issue #1989
// test('renders table with correct data', async () => {
// 	const queryClient = new QueryClient();
// 	const mockOrder: RaindexOrder = {
// 		id: '1',
// 		getVaultsVolume: vi.fn().mockResolvedValue({ value: mockVaultsVol })
// 	} as unknown as RaindexOrder;

// 	render(OrderVaultsVolTable, {
// 		context: new Map([['$$_queryClient', queryClient]]),
// 		props: { order: mockOrder }
// 	});

// 	await waitFor(async () => {
// 		// get total ins
// 		const rows = screen.getAllByTestId('total-in');

// 		// checking the total ins
// 		for (let i = 0; i < mockVaultsVol.length; i++) {
// 			const display = mockVaultsVol[i].details.formattedTotalIn;
// 			expect(rows[i]).toHaveTextContent(display);
// 		}
// 	});

// 	await waitFor(async () => {
// 		// get total outs
// 		const rows = screen.getAllByTestId('total-out');

// 		// checking the total outs
// 		for (let i = 0; i < mockVaultsVol.length; i++) {
// 			const display = mockVaultsVol[i].details.formattedTotalOut;
// 			expect(rows[i]).toHaveTextContent(display);
// 		}
// 	});

// 	await waitFor(async () => {
// 		// get net vols
// 		const rows = screen.getAllByTestId('net-vol');

// 		// checking the net vols
// 		for (let i = 0; i < mockVaultsVol.length; i++) {
// 			const display = mockVaultsVol[i].details.formattedNetVol;
// 			expect(rows[i]).toHaveTextContent(display);
// 		}
// 	});

// 	await waitFor(async () => {
// 		// get total vols
// 		const rows = screen.getAllByTestId('total-vol');

// 		// checking the total vols
// 		for (let i = 0; i < mockVaultsVol.length; i++) {
// 			const display = mockVaultsVol[i].details.formattedTotalVol;
// 			expect(rows[i]).toHaveTextContent(display);
// 		}
// 	});
// });
