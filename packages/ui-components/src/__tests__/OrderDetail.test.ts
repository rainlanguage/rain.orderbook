/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import OrderDetail from './OrderDetail.test.svelte';
import type { SgOrder, SgVault } from '@rainlanguage/orderbook/js_api';
import userEvent from '@testing-library/user-event';

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
				signerAddress: '0x123',
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

		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				signerAddress: '0x123',
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
		render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				signerAddress: 'notTheOwner',
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

		render(OrderDetail, {
			props: {
				orderHash: mockOrderWithVaults.orderHash,
				subgraphUrl: 'https://example.com',
				signerAddress: '0x123',
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
				signerAddress: '0x123',
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
	it('dispatches deposit event when deposit button is clicked', async () => {
		const user = userEvent.setup();
		const mockDispatch = vi.fn();

		const mockOrderWithVaults: SgOrder = {
			...mockOrder,
			inputs: [vault1]
		} as unknown as SgOrder;
		const sortedVaults = new Map([
			['inputs', [vault1]],
			['outputs', []]
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

		// Create component with a listener for the deposit event
		const { component } = render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				signerAddress: '0x123',
				chainId: 1,
				orderbookAddress
			}
		});

		// Mock the component's dispatch method
		component.$on('deposit', mockDispatch);

		// Wait for the component to finish loading data
		await waitFor(() => {
			expect(screen.queryByText('Order not found')).not.toBeInTheDocument();
		});

		// Find and click the deposit button
		const depositButton = await screen.getByTestId('deposit-button');
		await user.click(depositButton);

		// Verify dispatch was called with correct parameters
		expect(mockDispatch).toHaveBeenCalled();
		expect(mockDispatch.mock.calls[0][0].detail).toEqual({ vault: vault1 });
	});
	it('dispatches withdraw event when withdraw button is clicked', async () => {
		const user = userEvent.setup();
		const mockDispatch = vi.fn();

		const mockOrderWithVaults: SgOrder = {
			...mockOrder,
			outputs: [vault1]
		} as unknown as SgOrder;
		const sortedVaults = new Map([
			['inputs', []],
			['outputs', [vault1]]
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

		// Create component with a listener for the deposit event
		const { component } = render(OrderDetail, {
			props: {
				orderHash: 'mockHash',
				subgraphUrl: 'https://example.com',
				signerAddress: '0x123',
				chainId: 1,
				orderbookAddress
			}
		});

		// Mock the component's dispatch method
		component.$on('withdraw', mockDispatch);

		// Wait for the component to finish loading data
		await waitFor(() => {
			expect(screen.queryByText('Order not found')).not.toBeInTheDocument();
		});

		// Find and click the deposit button
		const withdrawButton = await screen.getByTestId('withdraw-button');
		await user.click(withdrawButton);

		// Verify dispatch was called with correct parameters
		expect(mockDispatch).toHaveBeenCalled();
		expect(mockDispatch.mock.calls[0][0].detail).toEqual({ vault: vault1 });
	});
});
