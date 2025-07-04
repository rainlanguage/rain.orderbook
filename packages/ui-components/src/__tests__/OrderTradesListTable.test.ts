import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { RaindexOrder, RaindexTrade } from '@rainlanguage/orderbook';
import { formatUnits } from 'viem';
import OrderTradesListTable from '../lib/components/tables/OrderTradesListTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

const mockTradeOrdersList: RaindexTrade[] = [
	{
		id: '1',
		timestamp: BigInt(1632000000),
		transaction: {
			id: 'transaction_id',
			from: '0xsender_address',
			timestamp: BigInt(1632000000),
			blockNumber: BigInt(0)
		},
		outputVaultBalanceChange: {
			amount: BigInt(-100),
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'output_token',
				symbol: 'output_token',
				decimals: '1'
			},
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			oldBalance: BigInt(0),
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderHash: 'orderHash',
		inputVaultBalanceChange: {
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'output_token',
				symbol: 'output_token',
				decimals: '1'
			},
			amount: BigInt(50),
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			oldBalance: BigInt(0),
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderbook: '0x00'
	},
	{
		id: '2',
		timestamp: BigInt(1632000000),
		transaction: {
			id: 'transaction_id',
			from: '0xsender_address',
			timestamp: BigInt(1632000000),
			blockNumber: BigInt(0)
		},
		outputVaultBalanceChange: {
			amount: BigInt(-100),
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'output_token',
				symbol: 'output_token',
				decimals: '1'
			},
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			oldBalance: BigInt(0),
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderHash: 'orderHash',
		inputVaultBalanceChange: {
			vaultId: BigInt(1),
			token: {
				id: 'output_token',
				address: '0xoutput_token',
				name: 'output_token',
				symbol: 'output_token',
				decimals: '1'
			},
			amount: BigInt(50),
			id: '1',
			__typename: 'Withdraw',
			newBalance: BigInt(0),
			oldBalance: BigInt(0),
			timestamp: BigInt(0),
			transaction: {
				id: 'transaction_id',
				from: '0xsender_address',
				timestamp: BigInt(1632000000),
				blockNumber: BigInt(0)
			},
			orderbook: '0x1'
		},
		orderbook: '0x00'
	}
] as unknown as RaindexTrade[];

vi.mock('@tanstack/svelte-query');

const mockOrder: RaindexOrder = {
	id: '1',
	getTradeCount: vi.fn(),
	getTradesList: vi.fn()
} as unknown as RaindexOrder;

test('renders table with correct data', async () => {
	const queryClient = new QueryClient();

	const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		subscribe: (fn: (value: any) => void) => {
			fn({
				data: { pages: [mockTradeOrdersList] },
				status: 'success',
				isFetching: false,
				isFetched: true
			});
			return { unsubscribe: () => {} };
		}
	})) as Mock;

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: { order: mockOrder, rpcUrl: 'https://example.com' }
	});

	await waitFor(async () => {
		// get all the io ratios
		const rows = screen.getAllByTestId('io-ratio');

		// checking the io ratios
		for (let i = 0; i < mockTradeOrdersList.length; i++) {
			const inputDisplay = formatUnits(
				BigInt(mockTradeOrdersList[i].inputVaultBalanceChange.amount),
				Number(mockTradeOrdersList[i].inputVaultBalanceChange.token.decimals)
			);
			const outputDisplay = formatUnits(
				BigInt(mockTradeOrdersList[i].outputVaultBalanceChange.amount),
				Number(mockTradeOrdersList[i].outputVaultBalanceChange.token.decimals)
			);
			const ioRatio = Number(inputDisplay) / (Number(outputDisplay) * -1);
			const oiRatio = (Number(outputDisplay) * -1) / Number(inputDisplay);
			expect(rows[i]).toHaveTextContent(ioRatio.toString());
			expect(rows[i]).toHaveTextContent(oiRatio.toString());
		}
	});
});

test('renders a debug button for each trade', async () => {
	const queryClient = new QueryClient();

	mockIPC((cmd) => {
		if (cmd === 'order_trades_list') {
			return mockTradeOrdersList;
		}
	});

	render(OrderTradesListTable, {
		context: new Map([['$$_queryClient', queryClient]]),
		props: {
			order: mockOrder,
			rpcUrl: 'example.com',
			handleDebugTradeModal: () => {}
		}
	});

	await waitFor(async () => {
		const buttons = screen.getAllByTestId('debug-trade-button');
		expect(buttons).toHaveLength(mockTradeOrdersList.length);
	});
});
