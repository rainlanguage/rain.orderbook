import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import ListViewOrderbookFilters from './ListViewOrderbookFilters.svelte';
import type { ConfigSource } from '../typeshare/config';

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

	const defaultProps = {
		settings: mockSettings,
		accounts: writable({}),
		hideZeroBalanceVaults: writable(false),
		activeAccountsItems: writable({}),
		activeSubgraphs: writable({}),
		activeOrderStatus: writable(undefined),
		orderHash: writable(''),
		isVaultsPage: false,
		isOrdersPage: false
	};

	beforeEach(() => {
		// Reset settings to default state before each test
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
		expect(screen.queryByTestId('accounts-dropdown')).not.toBeInTheDocument();
	});

	test('shows vault-specific components on vault page', () => {
		render(ListViewOrderbookFilters, {
			...defaultProps,
			isVaultsPage: true
		});

		expect(screen.getByTestId('zero-balance-vault-checkbox')).toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-dropdown')).not.toBeInTheDocument();
	});

	test('shows order-specific components on orders page', () => {
		render(ListViewOrderbookFilters, {
			...defaultProps,
			isOrdersPage: true
		});

		expect(screen.getByTestId('order-hash-input')).toBeInTheDocument();
		expect(screen.getByTestId('order-status-dropdown')).toBeInTheDocument();
		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
	});

	test('shows common components when networks exist', () => {
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.getByTestId('accounts-dropdown')).toBeInTheDocument();
		expect(screen.getByTestId('subgraphs-dropdown')).toBeInTheDocument();
	});

	test('does not show page-specific components on default view', () => {
		render(ListViewOrderbookFilters, defaultProps);

		expect(screen.queryByTestId('zero-balance-vault-checkbox')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-hash-input')).not.toBeInTheDocument();
		expect(screen.queryByTestId('order-status-dropdown')).not.toBeInTheDocument();
	});
});
