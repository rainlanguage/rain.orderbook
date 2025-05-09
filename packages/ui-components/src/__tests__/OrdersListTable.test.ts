/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock } from 'vitest';
import OrdersListTable from '../lib/components/tables/OrdersListTable.svelte';
import { readable } from 'svelte/store';
import type { SgOrderWithSubgraphName } from '@rainlanguage/orderbook';
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

const mockOrderWithSubgraph: SgOrderWithSubgraphName = {
	order: {
		id: '0x1234567890abcdef1234567890abcdef12345678',
		orderHash: '0x4444444444444444444444444444444444444444',
		owner: '0xabcdef1234567890abcdef1234567890abcdef12',
		active: true,
		orderbook: {
			id: '0x2222222222222222222222222222222222222222'
		},
		timestampAdded: '1678901234',
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
		trades: [
			{
				id: '0x5555555555555555555555555555555555555555'
			},
			{
				id: '0x6666666666666666666666666666666666666666'
			}
		]
	},
	subgraphName: 'mock-subgraph-mainnet'
} as SgOrderWithSubgraphName;

vi.mock('@tanstack/svelte-query');

// Hoisted mock stores
const {
	mockActiveNetworkRefStore,
	mockActiveOrderbookRefStore,
	mockHideZeroBalanceVaultsStore,
	mockOrderHashStore,
	mockAccountsStore,
	mockActiveAccountsItemsStore,
	mockshowInactiveOrdersStore,
	mockActiveSubgraphsStore,
	mockSettingsStore,
	mockShowMyItemsOnlyStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type OrdersListTableProps = ComponentProps<OrdersListTable<any>>;

const defaultProps: OrdersListTableProps = {
	activeSubgraphs: mockActiveSubgraphsStore,
	settings: mockSettingsStore,
	accounts: mockAccountsStore,
	activeAccountsItems: mockActiveAccountsItemsStore,
	showInactiveOrders: mockshowInactiveOrdersStore,
	orderHash: mockOrderHashStore,
	hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore,
	showMyItemsOnly: mockShowMyItemsOnlyStore,
	currentRoute: '/orders',
	activeNetworkRef: mockActiveNetworkRefStore,
	activeOrderbookRef: mockActiveOrderbookRefStore
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
					data: { pages: [[mockOrderWithSubgraph]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		expect(screen.getByTestId('orderListRowNetwork')).toHaveTextContent('mock-subgraph-mainnet');
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
					data: { pages: [[mockOrderWithSubgraph]] },
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
			const menuButton = screen.getByTestId(`order-menu-${mockOrderWithSubgraph.order.id}`);
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
					data: { pages: [[mockOrderWithSubgraph]] },
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

		const menuButton = screen.getByTestId(`order-menu-${mockOrderWithSubgraph.order.id}`);
		await userEvent.click(menuButton);

		const removeButton = screen.getByText('Remove');
		await userEvent.click(removeButton);

		expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockOrderWithSubgraph.order, mockRefetch);
	});

	it('shows inactive badge for inactive orders', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const inactiveOrder = {
			...mockOrderWithSubgraph,
			order: {
				...mockOrderWithSubgraph.order,
				active: false
			}
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
			...mockOrderWithSubgraph,
			order: {
				...mockOrderWithSubgraph.order,
				active: false
			}
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

		expect(
			screen.queryByTestId(`order-menu-${mockOrderWithSubgraph.order.id}`)
		).not.toBeInTheDocument();
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
					data: { pages: [[mockOrderWithSubgraph]] },
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
				item: mockOrderWithSubgraph
			}
		});

		// Find the AppTable component and dispatch the event
		const appTable = document.querySelector('div[role="table"]');
		if (appTable) {
			appTable.dispatchEvent(event);
			expect(gotoMock.goto).toHaveBeenCalledWith(
				`/orders/${mockOrderWithSubgraph.subgraphName}-${mockOrderWithSubgraph.order.orderHash}`
			);
		}
	});

	it('handles large number of trades display', async () => {
		mockMatchesAccount.mockReturnValue(true);

		const orderWithManyTrades = {
			...mockOrderWithSubgraph,
			order: {
				...mockOrderWithSubgraph.order,
				trades: Array(100).fill({
					id: '0x5555555555555555555555555555555555555555'
				})
			}
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
