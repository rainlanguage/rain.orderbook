import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { beforeEach, expect, test, describe, vi } from 'vitest';
import ListViewOrderbookFilters from '../lib/components/ListViewOrderbookFilters.svelte';
import type { ConfigSource } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';

vi.mock('$lib/providers/wallet/useAccount', () => ({
  useAccount: () => ({
    account: writable(null)
  })
}));

vi.mock('@tanstack/svelte-query', () => ({
  createInfiniteQuery: vi.fn()
}));

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ListViewOrderbookFiltersProps = ComponentProps<ListViewOrderbookFilters<any>>;

describe('ListViewOrderbookFilters', () => {
	const mockSettings = writable<ConfigSource>({
		networks: {
			ethereum: {
				rpc: 'https://rpc.ankr.com/eth',
				'chain-id': 1,
				'network-id': 1,
				currency: 'ETH'
			}
		},
		subgraphs: {
			mainnet: 'mainnet-url'
		}
	});

	const mockQuery = {
		data: { pages: [['page1']] },
		fetchNextPage: vi.fn(),
		hasNextPage: false,
		isFetchingNextPage: false,
		status: 'success',
		refetch: vi.fn()
	};

	const defaultProps = {
		settings: mockSettings,
		accounts: writable({}),
		hideZeroBalanceVaults: writable(false),
		activeAccountsItems: writable({}),
		activeSubgraphs: writable({}),
		activeOrderStatus: writable(undefined),
		orderHash: writable(''),
		isVaultsPage: false,
		isOrdersPage: false,
		showMyItemsOnly: writable(false)
	} as ListViewOrderbookFiltersProps;

	beforeEach(() => {
		mockSettings.set({
			networks: {
				ethereum: {
					rpc: 'https://rpc.ankr.com/eth',
					'chain-id': 1,
					'network-id': 1,
					currency: 'ETH'
				}
			},
			subgraphs: {
				mainnet: 'mainnet-url'
			}
		});
	});

	test('shows no networks alert when networks are empty', () => {
		mockSettings.set({ networks: {}, subgraphs: {} });
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('no-networks-alert')).toBeInTheDocument();
		expect(screen.queryByTestId('my-items-only')).not.toBeInTheDocument();
	});

	test('shows vault-specific components on vault page', () => {
		render(ListViewOrderbookFilters, {
			...defaultProps,
			isVaultsPage: true
		} as ListViewOrderbookFiltersProps);

		expect(screen.getByTestId('zero-balance-vault-checkbox')).toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-dropdown')).not.toBeInTheDocument();
	});

	test('shows order-specific components on orders page', () => {
		render(ListViewOrderbookFilters, {
			...defaultProps,
			isOrdersPage: true
		} as ListViewOrderbookFiltersProps);

		expect(screen.getByTestId('order-hash-input')).toBeInTheDocument();
		expect(screen.getByTestId('order-status-dropdown')).toBeInTheDocument();
		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
	});

	test('shows common components when networks exist', () => {
		const props = {
			...defaultProps,
			showMyItemsOnly: writable(true),
			activeAccountsItems: undefined,
			accounts: undefined
		};
		render(ListViewOrderbookFilters, props);

		expect(screen.getByTestId('subgraphs-dropdown')).toBeInTheDocument();
	});

	test('does not show page-specific components on default view', () => {
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-dropdown')).not.toBeInTheDocument();
	});
});