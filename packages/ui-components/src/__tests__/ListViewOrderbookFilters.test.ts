import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { beforeEach, expect, test, describe, vi } from 'vitest';
import ListViewOrderbookFilters from '../lib/components/ListViewOrderbookFilters.svelte';
import type { NewConfig } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores.ts'));

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: () => ({
		account: writable(null),
		matchesAccount: vi.fn()
	})
}));

vi.mock('$app/stores', () => ({
	page: mockPageStore
}));

vi.mock('@tanstack/svelte-query', () => ({
	createInfiniteQuery: vi.fn()
}));

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ListViewOrderbookFiltersProps = ComponentProps<ListViewOrderbookFilters<any>>;

describe('ListViewOrderbookFilters', () => {
	const mockSettings = writable<NewConfig>({
		orderbook: {
			version: '1',
			networks: {
				ethereum: {
					key: 'ethereum',
					rpcs: ['https://rpc.ankr.com/eth'],
					chainId: 1,
					networkId: 1,
					currency: 'ETH'
				}
			},
			subgraphs: {
				mainnet: {
					key: 'mainnet',
					url: 'mainnet-url'
				}
			}
		}
	} as unknown as NewConfig);

	const defaultProps: ListViewOrderbookFiltersProps = {
		settings: mockSettings,
		accounts: writable({}),
		hideZeroBalanceVaults: writable(false),
		activeAccountsItems: writable({}),
		activeSubgraphs: writable({}),
		showInactiveOrders: writable(true),
		orderHash: writable(''),
		showMyItemsOnly: writable(false)
	} as ListViewOrderbookFiltersProps;

	beforeEach(() => {
		mockSettings.set({
			orderbook: {
				networks: {
					ethereum: {
						key: 'ethereum',
						rpcs: ['https://rpc.ankr.com/eth'],
						chainId: 1,
						networkId: 1,
						currency: 'ETH'
					}
				},
				subgraphs: {
					mainnet: {
						key: 'mainnet',
						url: 'mainnet-url'
					}
				}
			}
		} as unknown as NewConfig);
	});

	test('shows no networks alert when networks are empty', () => {
		mockSettings.set({
			orderbook: {
				networks: {},
				subgraphs: {}
			}
		} as unknown as NewConfig);
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('no-networks-alert')).toBeInTheDocument();
		expect(screen.queryByTestId('my-items-only')).not.toBeInTheDocument();
	});

	test('shows vault-specific components on vault page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/vaults'
			} as URL
		});
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('zero-balance-vault-checkbox')).toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-checkbox')).not.toBeInTheDocument();
	});

	test('shows order-specific components on orders page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/orders'
			} as URL
		});
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('order-hash-input')).toBeInTheDocument();
		expect(screen.getByTestId('order-status-checkbox')).toBeInTheDocument();
		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
	});

	test('shows common components when networks exist', () => {
		const props = {
			...defaultProps,
			showMyItemsOnly: writable(true),
			activeAccountsItems: undefined,
			accounts: writable({})
		};
		render(ListViewOrderbookFilters, props);

		expect(screen.getByTestId('subgraphs-dropdown')).toBeInTheDocument();
	});

	test('does not show page-specific components on default view', () => {
		mockPageStore.reset();
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-checkbox')).not.toBeInTheDocument();
	});
});
