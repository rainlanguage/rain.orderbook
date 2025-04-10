import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { SgTrade } from '@rainlanguage/orderbook';
import { formatUnits } from 'viem';
import OrderTradesListTable from '../lib/components/tables/OrderTradesListTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

const mockTradeOrdersList: SgTrade[] = [
	{
		id: '1',
		timestamp: '1632000000',
		tradeEvent: {
			sender: 'sender_address',
			transaction: {
				id: 'transaction_id',
				from: 'sender_address',
				timestamp: '1632000000',
				blockNumber: '0'
			}
		},
		outputVaultBalanceChange: {
			amount: '-100',
			vault: {
				id: 'id',
				vault_id: 'vault-id',
				token: {
					id: 'output_token',
					address: 'output_token',
					name: 'output_token',
					symbol: 'output_token',
					decimals: '1'
				}
			},
			id: '1',
			__typename: 'Withdraw',
			newVaultBalance: '0',
			oldVaultBalance: '0',
			timestamp: '0',
			transaction: {
				id: 'transaction_id',
				from: 'sender_address',
				timestamp: '1632000000',
				blockNumber: '0'
			},
			orderbook: { id: '1' }
		},
		order: {
			id: 'order_id',
			orderHash: 'orderHash'
		},
		inputVaultBalanceChange: {
			vault: {
				id: 'id',
				vault_id: 'vault-id',
				token: {
					id: 'output_token',
					address: 'output_token',
					name: 'output_token',
					symbol: 'output_token',
					decimals: '1'
				}
			},
			amount: '50',
			id: '1',
			__typename: 'Withdraw',
			newVaultBalance: '0',
			oldVaultBalance: '0',
			timestamp: '0',
			transaction: {
				id: 'transaction_id',
				from: 'sender_address',
				timestamp: '1632000000',
				blockNumber: '0'
			},
			orderbook: { id: '1' }
		},
		orderbook: {
			id: '0x00'
		}
	},
	{
		id: '2',
		timestamp: '1632000000',
		tradeEvent: {
			sender: 'sender_address',
			transaction: {
				id: 'transaction_id',
				from: 'sender_address',
				timestamp: '1632000000',
				blockNumber: '0'
			}
		},
		outputVaultBalanceChange: {
			amount: '-100',
			vault: {
				id: 'id',
				vault_id: 'vault-id',
				token: {
					id: 'output_token',
					address: 'output_token',
					name: 'output_token',
					symbol: 'output_token',
					decimals: '1'
				}
			},
			id: '1',
			__typename: 'Withdraw',
			newVaultBalance: '0',
			oldVaultBalance: '0',
			timestamp: '0',
			transaction: {
				id: 'transaction_id',
				from: 'sender_address',
				timestamp: '1632000000',
				blockNumber: '0'
			},
			orderbook: { id: '1' }
		},
		order: {
			id: 'order_id',
			orderHash: 'orderHash'
		},
		inputVaultBalanceChange: {
			vault: {
				id: 'id',
				vault_id: 'vault-id',
				token: {
					id: 'output_token',
					address: 'output_token',
					name: 'output_token',
					symbol: 'output_token',
					decimals: '1'
				}
			},
			amount: '50',
			id: '1',
			__typename: 'Withdraw',
			newVaultBalance: '0',
			oldVaultBalance: '0',
			timestamp: '0',
			transaction: {
				id: 'transaction_id',
				from: 'sender_address',
				timestamp: '1632000000',
				blockNumber: '0'
			},
			orderbook: { id: '1' }
		},
		orderbook: {
			id: '0x00'
		}
	}
] as unknown as SgTrade[];

vi.mock('@tanstack/svelte-query');

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
		props: { id: '1', subgraphUrl: 'https://example.com' }
	});

	await waitFor(async () => {
		// get all the io ratios
		const rows = screen.getAllByTestId('io-ratio');

		// checking the io ratios
		for (let i = 0; i < mockTradeOrdersList.length; i++) {
			const inputDisplay = formatUnits(
				BigInt(mockTradeOrdersList[i].inputVaultBalanceChange.amount),
				Number(mockTradeOrdersList[i].inputVaultBalanceChange.vault.token.decimals)
			);
			const outputDisplay = formatUnits(
				BigInt(mockTradeOrdersList[i].outputVaultBalanceChange.amount),
				Number(mockTradeOrdersList[i].outputVaultBalanceChange.vault.token.decimals)
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
			id: '1',
			subgraphUrl: 'https://example.com',
			rpcUrl: 'example.com',
			handleDebugTradeModal: () => {}
		}
	});

	await waitFor(async () => {
		const buttons = screen.getAllByTestId('debug-trade-button');
		expect(buttons).toHaveLength(mockTradeOrdersList.length);
	});
});
