/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock } from 'vitest';
import VaultsListTable from '../lib/components/tables/VaultsListTable.svelte';
import { readable } from 'svelte/store';
import type { SgVaultWithSubgraphName } from '@rainlanguage/orderbook/js_api';
import { accountIsOwner } from '../lib/services/accountIsOwner';
import type { ComponentProps } from 'svelte';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';

vi.mock('$lib/services/accountIsOwner', () => ({
	accountIsOwner: vi.fn()
}));

const mockAccountStore = readable('0xabcdef1234567890abcdef1234567890abcdef12');

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

const mockVaultWithSubgraph: SgVaultWithSubgraphName = {
	vault: {
		id: '0x1234567890abcdef1234567890abcdef12345678',
		owner: '0xabcdef1234567890abcdef1234567890abcdef12',
		vaultId: '42',
		balance: '1000000000000000000', // 1 ETH in wei
		token: {
			id: '0x1111111111111111111111111111111111111111',
			address: '0x1111111111111111111111111111111111111111',
			name: 'Mock Token',
			symbol: 'MTK',
			decimals: '18'
		},
		orderbook: {
			id: '0x2222222222222222222222222222222222222222'
		},
		ordersAsOutput: [
			{
				id: '0x3333333333333333333333333333333333333333',
				orderHash: '0x4444444444444444444444444444444444444444',
				active: true
			}
		],
		ordersAsInput: [],
		balanceChanges: []
	},
	subgraphName: 'mock-subgraph-mainnet'
};

vi.mock('@tanstack/svelte-query');

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
	mockSettingsStore,
	mockActiveAccountsStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

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
	activeAccounts: mockActiveAccountsStore,
	currentRoute: '/vaults'
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type VaultsListTableProps = ComponentProps<VaultsListTable<any>>;

describe('VaultsListTable', () => {
	it('displays vault information correctly', async () => {
		(useAccount as Mock).mockReturnValue({
			account: mockAccountStore
		});
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVaultWithSubgraph]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);
		expect(screen.getByTestId('vault-network')).toHaveTextContent('mock-subgraph-mainnet');
		expect(screen.getByTestId('vault-token')).toHaveTextContent('Mock Token');
		expect(screen.getByTestId('vault-balance')).toHaveTextContent('1 MTK');
	});

	it('shows deposit/withdraw buttons when handlers are provided', async () => {
		(useAccount as Mock).mockReturnValue({
			account: mockAccountStore
		});
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();
		(accountIsOwner as Mock).mockReturnValue(true);
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVaultWithSubgraph]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		} as unknown as VaultsListTableProps);

		await waitFor(() => {
			const menuButton = screen.getByTestId('vault-menu');
			userEvent.click(menuButton);
			expect(screen.getByTestId('deposit-button')).toBeInTheDocument();
			expect(screen.getByTestId('withdraw-button')).toBeInTheDocument();
		});
	});

	it('shows new vault button when handleDepositGenericModal is provided', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVaultWithSubgraph]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		const handleDepositGenericModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositGenericModal
		} as unknown as VaultsListTableProps);

		expect(screen.getByTestId('new-vault-button')).toBeInTheDocument();
	});

	it('handles deposit action', async () => {
		(useAccount as Mock).mockReturnValue({
			account: mockAccountStore
		});
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		(accountIsOwner as Mock).mockReturnValue(true);
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVaultWithSubgraph]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		render(VaultsListTable, {
			...defaultProps,
			handleDepositModal,
			handleWithdrawModal
		} as unknown as VaultsListTableProps);

		const menuButton = screen.getByTestId('vault-menu');
		await userEvent.click(menuButton);

		const depositButton = screen.getByTestId('deposit-button');
		await userEvent.click(depositButton);

		expect(handleDepositModal).toHaveBeenCalledWith(mockVaultWithSubgraph.vault, undefined);
	});

	it('hides action buttons when user is not the vault owner', () => {
		vi.mocked(accountIsOwner).mockReturnValue(false);
		render(VaultsListTable, {
			...defaultProps
		} as unknown as VaultsListTableProps);

		expect(screen.queryByTestId('vault-menu')).not.toBeInTheDocument();
	});

	it('displays empty state when no vaults are found', async () => {
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

		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);
		expect(screen.getByText('No Vaults Found')).toBeInTheDocument();
	});
});
