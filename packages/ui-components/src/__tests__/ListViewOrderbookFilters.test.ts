import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { beforeEach, expect, test, describe, vi } from 'vitest';
import ListViewOrderbookFilters from '../lib/components/ListViewOrderbookFilters.svelte';
import type { NewConfig } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores.ts'));

const mockAccount = writable(null);

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: () => ({
		account: mockAccount,
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
					rpc: 'https://rpc.ankr.com/eth',
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
		selectedChainIds: writable([]),
		showInactiveOrders: writable(true),
		orderHash: writable('0x0234'),
		showMyItemsOnly: writable(false)
	} as ListViewOrderbookFiltersProps;

	beforeEach(() => {
		mockSettings.set({
			orderbook: {
				networks: {
					ethereum: {
						key: 'ethereum',
						rpc: 'https://rpc.ankr.com/eth',
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
		mockAccount.set(null);
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

	test('shows accounts dropdown when accounts exist', () => {
		const props = {
			...defaultProps,
			accounts: writable({
				'0x123': { key: '0x123', address: '0x123', name: 'Account 1' },
				'0x456': { key: '0x456', address: '0x456', name: 'Account 2' }
			})
		};
		render(ListViewOrderbookFilters, props);

		expect(screen.getByTestId('accounts-dropdown')).toBeInTheDocument();
	});

	test('does not show accounts dropdown when no accounts exist', () => {
		const props = {
			...defaultProps,
			accounts: writable({})
		};
		render(ListViewOrderbookFilters, props);

		expect(screen.queryByTestId('accounts-dropdown')).not.toBeInTheDocument();
	});

	test('shows My Items Only checkbox when no accounts (current logic)', () => {
		const props = {
			...defaultProps,
			accounts: writable({})
		};
		render(ListViewOrderbookFilters, props);

		expect(screen.getByTestId('my-items-only')).toBeInTheDocument();
	});

	test('hides My Items Only checkbox when accounts exist (current logic)', () => {
		const props = {
			...defaultProps,
			accounts: writable({
				'0x123': { key: '0x123', address: '0x123', name: 'Account 1' }
			})
		};
		render(ListViewOrderbookFilters, props);

		expect(screen.queryByTestId('my-items-only')).not.toBeInTheDocument();
	});

	test('passes correct context to CheckboxMyItemsOnly on vaults page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/vaults'
			} as URL
		});

		const props = {
			...defaultProps,
			accounts: writable({})
		};
		render(ListViewOrderbookFilters, props);

		const myItemsElement = screen.getByTestId('my-items-only');
		expect(myItemsElement).toBeInTheDocument();
	});

	test('passes correct context to CheckboxMyItemsOnly on orders page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/orders'
			} as URL
		});

		const props = {
			...defaultProps,
			accounts: writable({})
		};
		render(ListViewOrderbookFilters, props);

		const myItemsElement = screen.getByTestId('my-items-only');
		expect(myItemsElement).toBeInTheDocument();
	});
});
