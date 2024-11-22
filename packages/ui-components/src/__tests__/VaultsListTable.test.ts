import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import VaultsListTable from '../lib/components/tables/VaultsListTable.svelte';
import { writable, readable } from 'svelte/store';
import type { Vault } from '../lib/typeshare/subgraphTypes';

import { createResolvableInfiniteQuery } from '$lib/__mocks__/queries';



// Hoisted mock stores
const {
	mockActiveNetworkRefStore,
	mockActiveOrderbookRefStore,
	mockHideZeroBalanceVaultsStore,
	mockOrderHashStore,
	mockAccountsStore,
	mockActiveAccountsItemsStore,
	mockActiveOrderStatusStore,
	mockActiveSubgraphsStore,
	mockSettingsStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

const mockVault: Vault = {
	id: '1',
	vaultId: '1',
	owner: '0x123',
	token: {
		id: '0xtoken',
		name: 'Test Token',
		symbol: 'TEST',
		decimals: '18'
	},
	balance: '1000000000000000000',
	orderbook: {
		id: '0xorderbook'
	},
	ordersAsInput: [],
	ordersAsOutput: []
};

const mockVaultWithSubgraph = {
	vault: mockVault,
	subgraphName: 'testnet'
};

const defaultProps = {
	activeOrderbook: mockActiveOrderbookRefStore,
	subgraphUrl: readable('https://api.thegraph.com/subgraphs/name/test'),
	orderHash: mockOrderHashStore,
	accounts: mockAccountsStore,
	activeAccountsItems: mockActiveAccountsItemsStore,
	activeSubgraphs: mockActiveSubgraphsStore,
	settings: mockSettingsStore,
	activeOrderStatus: mockActiveOrderStatusStore,
	hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore,
	activeNetworkRef: mockActiveNetworkRefStore,
	activeOrderbookRef: mockActiveOrderbookRefStore,
	walletAddressMatchesOrBlank: readable(() => true),
	currentRoute: '/vaults'
};

describe('VaultsListTable', () => {
	beforeEach(() => {
		vi.mock('@tanstack/svelte-query', () => ({
			createInfiniteQuery: createResolvableInfiniteQuery(() => [
				{
					vault: {
						id: '0xabc',
						vaultId: '0xabc',
						owner: '0x123',
						token: {
							id: '1',
							address: '0x456',
							name: 'USDC coin',
							symbol: 'USDC',
							decimals: '6'
						},
						balance: '100000000000',
						ordersAsInput: [],
						ordersAsOutput: [],
						orderbook: { id: '0x00' }
					},
					subgraphName: 'network-one'
				}
			])
		}));
	});

	it('renders without crashing', () => {
		render(VaultsListTable, defaultProps);
		expect(screen.getByText('Vaults')).toBeInTheDocument();
	});

	it('displays vault information correctly', () => {


		render(VaultsListTable, defaultProps);
		expect(screen.getByTestId('vault-network')).toHaveTextContent('testnet');
		expect(screen.getByTestId('vault-token')).toHaveTextContent('Test Token');
		expect(screen.getByTestId('vault-balance')).toHaveTextContent('1.0 TEST');
	});

	it('shows deposit/withdraw buttons when handlers are provided', () => {
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		});

		const menuButton = screen.getByTestId('vault-menu');
		fireEvent.click(menuButton);

		expect(screen.getByTestId('deposit-button')).toBeInTheDocument();
		expect(screen.getByTestId('withdraw-button')).toBeInTheDocument();
	});

	it('shows new vault button when handleDepositGenericModal is provided', () => {
		const handleDepositGenericModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositGenericModal
		});

		expect(screen.getByTestId('new-vault-button')).toBeInTheDocument();
	});

	it('handles deposit action', async () => {
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		});

		const menuButton = screen.getByTestId('vault-menu');
		fireEvent.click(menuButton);

		const depositButton = screen.getByTestId('deposit-button');
		fireEvent.click(depositButton);

		expect(handleDepositModal).toHaveBeenCalledWith(mockVault, expect.any(Function));
	});

	it('handles withdraw action', async () => {
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		});

		const menuButton = screen.getByTestId('vault-menu');
		fireEvent.click(menuButton);

		const withdrawButton = screen.getByTestId('withdraw-button');
		fireEvent.click(withdrawButton);

		expect(handleWithdrawModal).toHaveBeenCalledWith(mockVault, expect.any(Function));
	});

	it('hides action buttons when user is not the vault owner', () => {
		render(VaultsListTable, {
			...defaultProps,
			walletAddressMatchesOrBlank: readable(() => false)
		});

		expect(screen.queryByTestId('vault-menu')).not.toBeInTheDocument();
	});

	it('displays empty state when no vaults are found', () => {
		vi.mock('@tanstack/svelte-query', () => ({
			createInfiniteQuery: () => ({
				subscribe: (fn: (value: any) => void) => {
					fn({
						data: { pages: [[]] },
						fetchNextPage: vi.fn(),
						hasNextPage: false,
						isFetchingNextPage: false,
						refetch: vi.fn()
					});
					return { unsubscribe: vi.fn() };
				}
			})
		}));

		render(VaultsListTable, defaultProps);
		expect(screen.getByText('No Vaults Found')).toBeInTheDocument();
	});
});
