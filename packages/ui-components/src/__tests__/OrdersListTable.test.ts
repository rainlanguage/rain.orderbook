/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock } from 'vitest';
import OrdersListTable from '../lib/components/tables/OrdersListTable.svelte';
import { readable } from 'svelte/store';
import { RaindexOrder } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';

vi.mock('../lib/components/ListViewOrderbookFilters.svelte', async () => {
	const MockComponent = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return {
		default: MockComponent
	};
});

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

const mockAccountStore = readable('0xabcdef1234567890abcdef1234567890abcdef12');

const mockOrder = {
	chainId: 1,
	id: '0x1234567890abcdef1234567890abcdef12345678',
	orderBytes: '',
	orderHash: '0x4444444444444444444444444444444444444444',
	owner: '0xabcdef1234567890abcdef1234567890abcdef12',
	inputs: [
		{
			token: {
				symbol: 'ETH'
			}
		}
	],
	outputs: [
		{
			token: {
				symbol: 'DAI'
			}
		}
	],
	vaults: [],
	orderbook: '0x2222222222222222222222222222222222222222',
	active: true,
	timestampAdded: BigInt(1678901234),
	meta: '',
	rainlang: '',
	tradesCount: 2
} as unknown as RaindexOrder;

vi.mock('@tanstack/svelte-query');

// Hoisted mock stores
const {
	mockActiveNetworkRefStore,
	mockActiveOrderbookRefStore,
	mockHideZeroBalanceVaultsStore,
	mockOrderHashStore,
	mockAccountsStore,
	mockActiveAccountsItemsStore,
	mockShowInactiveOrdersStore,
	mockShowMyItemsOnlyStore,
	mockSelectedChainIdsStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type OrdersListTableProps = ComponentProps<OrdersListTable<any>>;

const defaultProps: OrdersListTableProps = {
	accounts: mockAccountsStore,
	activeAccountsItems: mockActiveAccountsItemsStore,
	showInactiveOrders: mockShowInactiveOrdersStore,
	orderHash: mockOrderHashStore,
	hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore,
	showMyItemsOnly: mockShowMyItemsOnlyStore,
	activeNetworkRef: mockActiveNetworkRefStore,
	activeOrderbookRef: mockActiveOrderbookRefStore,
	selectedChainIds: mockSelectedChainIdsStore
} as unknown as OrdersListTableProps;

const mockMatchesAccount = vi.fn();

describe('OrdersListTable', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useAccount as Mock).mockReturnValue({
			account: mockAccountStore,
			matchesAccount: mockMatchesAccount
		});
	});

	it('displays order information correctly', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockOrder]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		expect(screen.getByTestId('orderListRowNetwork')).toHaveTextContent('Ethereum');
		expect(screen.getByTestId('orderListRowActive')).toHaveTextContent('Active');
		expect(screen.getByTestId('orderListRowInputs')).toHaveTextContent('ETH');
		expect(screen.getByTestId('orderListRowOutputs')).toHaveTextContent('DAI');
		expect(screen.getByTestId('orderListRowTrades')).toHaveTextContent('2');
	});

	it('shows remove button when order is active and user is owner', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockOrder]] },
					status: 'success',
					isFetching: false,
					isFetched: true,
					refetch: vi.fn()
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, {
			...defaultProps,
			handleOrderRemoveModal: vi.fn()
		} as OrdersListTableProps);

		await waitFor(() => {
			const menuButton = screen.getByTestId(`order-menu-${mockOrder.id}`);
			userEvent.click(menuButton);
			expect(screen.getByText('Remove')).toBeInTheDocument();
		});
	});

	it('handles remove action', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		const mockRefetch = vi.fn();
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockOrder]] },
					status: 'success',
					isFetching: false,
					isFetched: true,
					refetch: mockRefetch
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		const handleOrderRemoveModal = vi.fn();

		render(OrdersListTable, {
			...defaultProps,
			handleOrderRemoveModal
		} as OrdersListTableProps);

		const menuButton = screen.getByTestId(`order-menu-${mockOrder.id}`);
		await userEvent.click(menuButton);

		const removeButton = screen.getByText('Remove');
		await userEvent.click(removeButton);

		expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockOrder, mockRefetch);
	});

	it('shows inactive badge for inactive orders', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const inactiveOrder = {
			...mockOrder,
			active: false
		};
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[inactiveOrder]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		expect(screen.getByTestId('orderListRowActive')).toHaveTextContent('Inactive');
	});

	it('does not show action menu for inactive orders', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const inactiveOrder = {
			...mockOrder,
			active: false
		};
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[inactiveOrder]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		const handleOrderRemoveModal = vi.fn();

		render(OrdersListTable, {
			...defaultProps,
			handleOrderRemoveModal
		} as OrdersListTableProps);

		expect(screen.queryByTestId(`order-menu-${mockOrder.id}`)).not.toBeInTheDocument();
	});

	it('displays empty state when no orders are found', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);
		expect(screen.getByText('No Orders Found')).toBeInTheDocument();
	});

	it('navigates to order details on row click', async () => {
		vi.mock('$app/navigation', () => ({
			goto: vi.fn()
		}));

		const gotoMock = await import('$app/navigation');
		mockMatchesAccount.mockReturnValue(true);
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockOrder]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Simulate row click
		const event = new CustomEvent('clickRow', {
			detail: {
				item: mockOrder
			}
		});

		// Find the AppTable component and dispatch the event
		const appTable = document.querySelector('div[role="table"]');
		if (appTable) {
			appTable.dispatchEvent(event);
			expect(gotoMock.goto).toHaveBeenCalledWith(
				`/orders/${mockOrder.chainId}-${mockOrder.orderbook}-${mockOrder.orderHash}`
			);
		}
	});

	it('handles large number of trades display', async () => {
		mockMatchesAccount.mockReturnValue(true);

		const orderWithManyTrades = {
			...mockOrder,
			tradesCount: 100
		};

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[orderWithManyTrades]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrdersListTable, defaultProps as OrdersListTableProps);
		expect(screen.getByTestId('orderListRowTrades')).toHaveTextContent('>99');
	});
});
