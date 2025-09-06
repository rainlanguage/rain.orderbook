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

const mockVaultsList = () => ({
	items: [],
	getWithdrawableVaults: () => ({ value: [], error: null })
});

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
			},
			formattedBalance: '1.5'
		}
	],
	outputs: [
		{
			token: {
				symbol: 'DAI'
			},
			formattedBalance: '2500.0'
		}
	],
	vaults: [],
	inputsList: {
		...mockVaultsList(),
		items: [
			{
				token: {
					symbol: 'ETH'
				},
				formattedBalance: '1.5'
			}
		]
	},
	outputsList: {
		...mockVaultsList(),
		items: [
			{
				token: {
					symbol: 'DAI'
				},
				formattedBalance: '2500.0'
			}
		]
	},
	inputsOutputsList: mockVaultsList(),
	vaultsList: mockVaultsList(),
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
	mockActiveAccountsItemsStore,
	mockShowInactiveOrdersStore,
	mockShowMyItemsOnlyStore,
	mockSelectedChainIdsStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

type OrdersListTableProps = ComponentProps<OrdersListTable>;

const defaultProps: OrdersListTableProps = {
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

		// Check that vault cards are rendered with correct content
		const vaultCards = screen.getAllByTestId('vault-card');
		expect(vaultCards).toHaveLength(2); // One input, one output
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();

		expect(screen.getByTestId('orderListRowTrades')).toHaveTextContent('2');
	});

	it('displays token information in compact layout', async () => {
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

		// Verify token symbols and balances are displayed in vault cards
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();

		// Verify "Strategy Balance:" label is not present (since we're using vault cards now)
		expect(screen.queryByText('Strategy Balance:')).not.toBeInTheDocument();
	});

	it('displays multiple tokens correctly in grid layout with shared IO', async () => {
		const orderWithMultipleTokens = {
			...mockOrder,
			inputs: [
				{
					token: { symbol: 'ETH' },
					formattedBalance: '1.5'
				},
				{
					token: { symbol: 'USDC' },
					formattedBalance: '100.0'
				}
			],
			outputs: [
				{
					token: { symbol: 'DAI' },
					formattedBalance: '2500.0'
				},
				{
					token: { symbol: 'USDC' },
					formattedBalance: '100.0'
				}
			],
			inputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'ETH', address: '0xeth' },
						formattedBalance: '1.5'
					},
				]
			},
			outputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'DAI', address: '0xdai' },
						formattedBalance: '2500.0'
					},
				]
			},
			inputsOutputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'USDC', address: '0xusdc' },
						formattedBalance: '100.0'
					}
				]
			}
		};

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[orderWithMultipleTokens]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Verify all tokens are displayed in vault cards
		const vaultCards = screen.getAllByTestId('vault-card');
		expect(vaultCards).toHaveLength(4); // 2 inputs + 2 outputs (1 is shared between IO)

		// Verify all input tokens are displayed
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();

		// Verify all output tokens are displayed
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();

		// Verify shared token is displayed for input and output
		expect(screen.getAllByText('USDC')).toHaveLength(2);
		expect(screen.getAllByText('100.0')).toHaveLength(2);
	});

	it('displays multiple tokens correctly in grid layout', async () => {
		const orderWithMultipleTokens = {
			...mockOrder,
			inputs: [
				{
					token: { symbol: 'ETH' },
					formattedBalance: '1.5'
				},
				{
					token: { symbol: 'USDC' },
					formattedBalance: '100.0'
				}
			],
			outputs: [
				{
					token: { symbol: 'DAI' },
					formattedBalance: '2500.0'
				},
				{
					token: { symbol: 'WBTC' },
					formattedBalance: '0.05'
				}
			],
			inputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'ETH' },
						formattedBalance: '1.5'
					},
					{
						token: { symbol: 'USDC' },
						formattedBalance: '100.0'
					}
				]
			},
			outputsList: {
				...mockVaultsList(),
				items: [
					{
						token: { symbol: 'DAI' },
						formattedBalance: '2500.0'
					},
					{
						token: { symbol: 'WBTC' },
						formattedBalance: '0.05'
					}
				]
			}
		};

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[orderWithMultipleTokens]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(OrdersListTable, defaultProps as OrdersListTableProps);

		// Verify all tokens are displayed in vault cards
		const vaultCards = screen.getAllByTestId('vault-card');
		expect(vaultCards).toHaveLength(4); // 2 inputs + 2 outputs

		// Verify all input tokens are displayed
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
		expect(screen.getByText('USDC')).toBeInTheDocument();
		expect(screen.getByText('100.0')).toBeInTheDocument();

		// Verify all output tokens are displayed
		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();
		expect(screen.getByText('WBTC')).toBeInTheDocument();
		expect(screen.getByText('0.05')).toBeInTheDocument();
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

		expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockOrder, mockRefetch, new Map());
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
