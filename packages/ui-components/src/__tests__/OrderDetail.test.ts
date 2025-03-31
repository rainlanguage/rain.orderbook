/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, beforeEach, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import OrderDetail from './OrderDetail.test.svelte';
import type { SgOrder, SgVault } from '@rainlanguage/orderbook/js_api';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';
import { writable } from 'svelte/store';
import { invalidateIdQuery } from '$lib/queries/queryClient';

vi.mock('$lib/queries/queryClient', () => ({
	invalidateIdQuery: vi.fn()
}));

// Mock wallet and wagmi config stores
const { mockWalletAddressMatchesOrBlankStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

// Mock useAccount
vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

// Mock tanstack/svelte-query
vi.mock('@tanstack/svelte-query');

// Create mock order data
const mockOrder: SgOrder = {
	id: 'mockId',
	owner: '0x123',
	orderHash: 'mockOrderHash',
	active: true,
	meta: '0x',
	timestampAdded: '1234567890',
	orderbook: { id: '1' },
	inputs: [],
	outputs: []
} as unknown as SgOrder;

// Test parameters
const chainId = 1;
const orderbookAddress = '0x123';
const rpcUrl = 'https://example.com';
const subgraphUrl = 'https://example.com';

describe('OrderDetail Component', () => {
	beforeEach(() => {
		// Reset the account mock before each test
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123')
		});
	});

	it('shows the correct empty message when the query returns no data', async () => {
		// Mock query to return null data
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

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		// Verify empty message
		await waitFor(() => expect(screen.getByText('Order not found')).toBeInTheDocument());
	});

	it('shows the order details when data is available', async () => {
		// Mock query to return order data
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

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		// Verify order details are displayed
		await waitFor(() => {
			expect(screen.getAllByText(/Order/)).toHaveLength(2);
			expect(screen.getAllByText(/mockOrderHash/)).toHaveLength(2);
		});
	});

	it('shows remove button if account matches order owner and order is active', async () => {
		// Setup account to match order owner
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123')
		});

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

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal,
				wagmiConfig: mockWagmiConfigStore
			}
		});

		// Verify remove button is displayed
		await waitFor(() => {
			expect(screen.getByTestId('remove-button')).toBeInTheDocument();
			expect(screen.getByText('Remove')).toBeInTheDocument();
		});

		// Verify button is clickable and calls handler
		await userEvent.click(screen.getByTestId('remove-button'));
		expect(handleOrderRemoveModal).toHaveBeenCalled();
		expect(handleOrderRemoveModal).toHaveBeenCalledWith({
			open: true,
			args: {
				order: mockOrder,
				onRemove: expect.any(Function),
				chainId,
				orderbookAddress,
				subgraphUrl
			}
		});
	});

	it('does not show remove button if account does not match order owner', async () => {
		// Setup account that doesn't match order owner
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x1234')
		});

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

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		// Verify remove button is not displayed
		await waitFor(() => {
			expect(screen.queryByText('Remove')).not.toBeInTheDocument();
		});
	});

	it('does not show remove button if order is not active', async () => {
		// Create inactive order
		const inactiveOrder = { ...mockOrder, active: false };

		// Setup account to match order owner
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123')
		});

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { order: inactiveOrder, vaults: new Map() },
					status: 'success',
					isFetching: false,
					refetch: () => {}
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		// Verify remove button is not displayed
		await waitFor(() => {
			expect(screen.queryByText('Remove')).not.toBeInTheDocument();
		});
	});

	it('correctly categorizes and displays vaults in input, output, and shared categories', async () => {
		// Create mock vaults
		const vault1 = {
			id: '1',
			orderHash: 'mockHash',
			vaultId: '0xabc',
			owner: 'mockOwner',
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
			owner: 'mockOwner',
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
			owner: 'mockOwner',
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

		// Create order with vaults
		const mockOrderWithVaults: SgOrder = {
			...mockOrder,
			inputs: [vault1, vault2],
			outputs: [vault2, vault3]
		} as unknown as SgOrder;

		// Sort vaults for display
		const sortedVaults = new Map([
			['inputs', [vault1]],
			['outputs', [vault3]],
			['inputs_outputs', [vault2]]
		]);

		// Setup account to match vault owner
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123')
		});

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
					isFetching: false,
					refetch: () => {}
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		// Verify vault categories are displayed
		await waitFor(() => {
			expect(screen.getByText('Input vaults')).toBeInTheDocument();
			expect(screen.getByText('Output vaults')).toBeInTheDocument();
			expect(screen.getByText('Input & output vaults')).toBeInTheDocument();
		});
	});

	it('shows deposit and withdraw buttons when account matches vault owner', async () => {
		// Create vault with matching owner
		const vault = {
			id: '1',
			orderHash: 'mockHash',
			vaultId: '0xabc',
			owner: 'mockOwner',
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

		// Set up sorted vaults
		const sortedVaults = new Map([['inputs', [vault]]]);

		// Create order with vault
		const orderWithVault = {
			...mockOrder,
			inputs: [vault]
		} as unknown as SgOrder;

		// Setup account to match vault owner
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123')
		});

		const handleDepositOrWithdrawModal = vi.fn();
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));

		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: {
						order: orderWithVault,
						vaults: sortedVaults
					},
					status: 'success',
					isFetching: false,
					refetch: () => {}
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal,
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		await waitFor(() => {
			expect(handleDepositOrWithdrawModal).toBeDefined();
			expect(handleDepositOrWithdrawModal).not.toHaveBeenCalled();
		});
	});

	it('refresh button triggers query invalidation when clicked', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));

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

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		const refreshButton = screen.getByTestId('refresh-button');
		await userEvent.click(refreshButton);

		await waitFor(() => {
			expect(invalidateIdQuery).toHaveBeenCalled();
		});
	});

	it('displays loading state when query is fetching', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));

		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: null,
					status: 'loading',
					isFetching: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		vi.mock('$lib/queries/queryClient', () => ({
			invalidateIdQuery: vi.fn()
		}));

		// Render component
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		await waitFor(() => {
			expect(mockQuery.createQuery).toHaveBeenCalled();
		});
	});

	it('handles error state in query response', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));

		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: null,
					status: 'error',
					error: new Error('Test error'),
					isFetching: false
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				chainId,
				orderbookAddress,
				rpcUrl,
				handleDepositOrWithdrawModal: vi.fn(),
				handleOrderRemoveModal: vi.fn(),
				wagmiConfig: mockWagmiConfigStore
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Order not found')).toBeInTheDocument();
		});
	});
});
