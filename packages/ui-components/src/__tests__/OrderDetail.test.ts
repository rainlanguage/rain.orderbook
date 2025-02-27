/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import OrderDetail from './OrderDetail.test.svelte';
import type { SgOrder, SgVault } from '@rainlanguage/orderbook/js_api';
import userEvent from '@testing-library/user-event';

const { mockWalletAddressMatchesOrBlankStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

const mockOrder: SgOrder = {
	id: 'mockId',
	owner: 'mockOwner',
	orderHash: 'mockOrderHash',
	active: true,
	meta: '0x',
	timestampAdded: '1234567890',
	orderbook: { id: '1' },
	inputs: [],
	outputs: []
} as unknown as SgOrder;

vi.mock('@tanstack/svelte-query');

const chainId = 1;
const orderbookAddress = '0x123';

describe('OrderDetail Component', () => {
	it('shows the correct empty message when the query returns no data', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: null,
					status: 'success',
					isFetching: false
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress
			}
		});

		await waitFor(() => expect(screen.getByText('Order not found')).toBeInTheDocument());
	});

	it('shows remove button if owner wallet matches and order is active', async () => {
		const handleOrderRemoveModal = vi.fn();
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { order: mockOrder, vaults: new Map() },
					status: 'success',
					isFetching: false,
					refetch: () => {}
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		mockWalletAddressMatchesOrBlankStore.mockSetSubscribeValue(() => true);

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				handleOrderRemoveModal,
				chainId,
				orderbookAddress
			}
		});

		await waitFor(() => {
			expect(screen.queryByText('Remove')).toBeInTheDocument();
			expect(handleOrderRemoveModal).not.toHaveBeenCalled();
		});
	});

	it('does not render the remove button if conditions are not met', async () => {
		mockWalletAddressMatchesOrBlankStore.mockSetSubscribeValue(() => false);

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				handleOrderRemoveModal: vi.fn(),
				chainId,
				orderbookAddress
			}
		});

		await waitFor(() => {
			expect(screen.queryByText('Remove')).not.toBeInTheDocument();
		});
	});

	it('correctly categorizes and displays vaults in input, output, and shared categories', async () => {
		const vault1 = {
			id: '1',
			orderHash: 'mockHash',
			vaultId: '0xabc',
			owner: '0x123',
			token: {
				id: '0x456',
				address: '0x456',
				name: 'USDC coin',
				symbol: 'USDC',
				decimals: '6'
			},
			balance: '100000000000',
			ordersAsInput: [],
			ordersAsOutput: [],
			balanceChanges: [],
			orderbook: {
				id: '0x00'
			}
		} as unknown as SgVault;
		const vault2 = {
				id: '2',
			orderHash: 'mockHash',
			vaultId: '0xbcd',
			owner: '0x123',
			token: {
				id: '0x456',
				address: '0x456',
				name: 'USDC coin',
				symbol: 'USDC',
				decimals: '6'
			},
			balance: '100000000000',
			ordersAsInput: [],
			ordersAsOutput: [],
			balanceChanges: [],
			orderbook: {
				id: '0x00'
			}
		} as unknown as SgVault;
		const vault3 = {
			id: '3',
			vaultId: '0xdef',
			owner: '0x123',
			orderHash: 'mockHash',
			token: {
				id: '0x456',
				address: '0x456',
				name: 'USDC coin',
				symbol: 'USDC',
				decimals: '6'
			},
			balance: '100000000000',
			ordersAsInput: [],
			ordersAsOutput: [],
			balanceChanges: [],
			orderbook: {
				id: '0x00'
			}
		} as unknown as SgVault;
		const mockOrderWithVaults: SgOrder = {
			...mockOrder,
			inputs: [vault1, vault2],
			outputs: [vault2, vault3]
		} as unknown as SgOrder;
		const sortedVaults = new Map([
			['inputs', [vault1]],
			['outputs', [vault3]],
			['inputs_outputs', [vault2]]
		]);

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: {
						order: mockOrderWithVaults,
						vaults: sortedVaults
					},
					status: 'success',
					isFetching: false
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		mockWalletAddressMatchesOrBlankStore.mockSetSubscribeValue(() => true);
		render(OrderDetail, {
			props: {
				orderHash: mockOrderWithVaults.orderHash,
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Input vaults')).toBeInTheDocument();
			expect(screen.getByText('Output vaults')).toBeInTheDocument();
			expect(screen.getByText('Input & output vaults')).toBeInTheDocument();
		});
	});

	it('refresh button triggers query invalidation when clicked', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		const mockInvalidateQueries = vi.fn();

		// Mock the createQuery as in other tests
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { order: mockOrder, vaults: new Map() },
					status: 'success',
					isFetching: false,
					refetch: () => {}
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		// Mock the useQueryClient hook
		mockQuery.useQueryClient = vi.fn(() => ({
			invalidateQueries: mockInvalidateQueries
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		})) as any;

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress
			}
		});

		const refreshButton = screen.getByTestId('refresh-button');
		await userEvent.click(refreshButton);

		await waitFor(() => {
			expect(mockInvalidateQueries).toHaveBeenCalledWith({
				queryKey: ['mockHash'],
				refetchType: 'all',
				exact: false
			});
		});
	});
});


