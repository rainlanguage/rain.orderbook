import { render, screen } from '@testing-library/svelte';
import { readable, writable } from 'svelte/store';
import { beforeEach, expect, test, describe, vi, type Mock } from 'vitest';
import ListViewOrderbookFilters from '../lib/components/ListViewOrderbookFilters.svelte';
import type { RaindexVaultToken } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import type { QueryObserverResult } from '@tanstack/svelte-query';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

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

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

type ListViewOrderbookFiltersProps = ComponentProps<ListViewOrderbookFilters>;

describe('ListViewOrderbookFilters', () => {
	const mockGetAllAccounts = vi.fn();

	const defaultProps: ListViewOrderbookFiltersProps = {
		hideZeroBalanceVaults: writable(false),
		activeAccountsItems: writable({}),
		selectedChainIds: writable([]),
		showInactiveOrders: writable(true),
		orderHash: writable('0x0234'),
		showMyItemsOnly: writable(false),
		activeTokens: writable([]),
		selectedTokens: [],
		tokensQuery: readable({
			isLoading: false,
			isError: false,
			data: [] as RaindexVaultToken[],
			error: null
		} as QueryObserverResult<RaindexVaultToken[], Error>)
	} as ListViewOrderbookFiltersProps;

	beforeEach(() => {
		(useRaindexClient as Mock).mockReturnValue({
			getUniqueChainIds: vi.fn(() => ({
				value: [1],
				error: undefined
			})),
			getAllNetworks: vi.fn(() => ({
				value: new Map([
					[
						'ethereum',
						{
							key: 'ethereum',
							rpcs: ['https://rpc.ankr.com/eth'],
							chainId: 1,
							networkId: 1,
							currency: 'ETH'
						}
					]
				]),
				error: undefined
			})),
			getAllAccounts: mockGetAllAccounts
		});

		// Set default return value for getAllAccounts
		mockGetAllAccounts.mockReturnValue({
			value: new Map(),
			error: undefined
		});

		mockAccount.set(null);
	});

	test('shows no networks alert when networks are empty', () => {
		(useRaindexClient as Mock).mockReturnValue({
			getAllNetworks: vi.fn(() => ({
				value: new Map(),
				error: undefined
			})),
			getAllAccounts: vi.fn(() => ({
				value: new Map(),
				error: undefined
			}))
		});
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
			activeAccountsItems: undefined
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
		mockGetAllAccounts.mockReturnValue({
			value: new Map([
				['0x123', { key: '0x123', address: '0x123', name: 'Account 1' }],
				['0x456', { key: '0x456', address: '0x456', name: 'Account 2' }]
			]),
			error: undefined
		});

		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('accounts-dropdown')).toBeInTheDocument();
	});

	test('does not show accounts dropdown when no accounts exist', () => {
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.queryByTestId('accounts-dropdown')).not.toBeInTheDocument();
	});

	test('shows My Items Only checkbox when no accounts (current logic)', () => {
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('my-items-only')).toBeInTheDocument();
	});

	test('hides My Items Only checkbox when accounts exist (current logic)', () => {
		mockGetAllAccounts.mockReturnValue({
			value: new Map([['0x123', { key: '0x123', address: '0x123', name: 'Account 1' }]]),
			error: undefined
		});

		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.queryByTestId('my-items-only')).not.toBeInTheDocument();
	});

	test('passes correct context to CheckboxMyItemsOnly on vaults page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/vaults'
			} as URL
		});

		render(ListViewOrderbookFilters, defaultProps);

		const myItemsElement = screen.getByTestId('my-items-only');
		expect(myItemsElement).toBeInTheDocument();
	});

	test('passes correct context to CheckboxMyItemsOnly on orders page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/orders'
			} as URL
		});

		render(ListViewOrderbookFilters, defaultProps);

		const myItemsElement = screen.getByTestId('my-items-only');
		expect(myItemsElement).toBeInTheDocument();
	});
});
