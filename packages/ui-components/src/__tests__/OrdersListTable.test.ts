import { render, screen } from '@testing-library/svelte';
import OrdersListTable from '../lib/components/tables/OrdersListTable.svelte';
import type { ComponentProps } from 'svelte';

const {
	mockSettingsStore,
	mockActiveSubgraphsStore,
	mockAccountsStore,
	mockActiveAccountsItemsStore,
	mockActiveOrderStatusStore,
	mockOrderHashStore,
	mockHideZeroBalanceVaultsStore
} = await vi.hoisted(() => import('../__mocks__/stores'));

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type OrdersListTableProps = ComponentProps<OrdersListTable<any>>;

vi.mock('@tanstack/svelte-query', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		createInfiniteQuery: vi.fn()
	};
});

vi.mock('../ListViewOrderbookFilters.svelte', async () => {
	return {
		default: await import('../__mocks__/MockListViewOrderbookFilters.svelte')
	};
});

vi.mock('../TanstackAppTable.svelte', async () => {
	return {
		default: await import('../__mocks__/MockTanstackAppTable.svelte')
	};
});

describe('OrdersListTable', () => {
	const mockStores = {
		settings: mockSettingsStore,
		activeSubgraphs: mockActiveSubgraphsStore,
		accounts: mockAccountsStore,
		activeAccountsItems: mockActiveAccountsItemsStore,
		activeOrderStatus: mockActiveOrderStatusStore,
		orderHash: mockOrderHashStore,
		hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore
	};

	const defaultProps = {
		...mockStores,
		currentRoute: '/orders'
	};

	it('shows filters when on orders page', async () => {
		render(OrdersListTable, {
			props: {
				...defaultProps,
				currentRoute: '/orders'
			} as unknown as OrdersListTableProps
		});

		expect(screen.getByTestId('orderbook-filters')).toBeInTheDocument();
		expect(screen.getByTestId('network-filter')).toBeInTheDocument();
		expect(screen.getByTestId('status-filter')).toBeInTheDocument();
	});

	it('shows filters when on vaults page', () => {
		render(OrdersListTable, {
			props: {
				...defaultProps,
				currentRoute: '/vaults'
			} as unknown as OrdersListTableProps
		});

		expect(screen.getByTestId('orderbook-filters')).toBeInTheDocument();
		expect(screen.getByTestId('zero-balance-filter')).toBeInTheDocument();
	});

	it('does not show remove option when wallet does not match', () => {
		const mockWalletAddressMatchesOrBlank = vi.fn().mockReturnValue(false);

		render(OrdersListTable, {
			props: {
				...defaultProps,
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlank
			} as unknown as OrdersListTableProps
		});

		expect(screen.queryByTestId('wallet-actions')).not.toBeInTheDocument();
	});
});
