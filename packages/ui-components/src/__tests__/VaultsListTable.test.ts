/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, type Mock } from 'vitest';
import VaultsListTable from '../lib/components/tables/VaultsListTable.svelte';
import { readable } from 'svelte/store';
import type { RaindexVault } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

const mockMatchesAccount = vi.fn();
const mockAccountStore = readable('0xabcdef1234567890abcdef1234567890abcdef12');

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: () => ({
		getUniqueChainIds: vi.fn(() => ({
			value: [1],
			error: undefined
		})),
		getAllNetworks: vi.fn(() => ({
			value: new Map([[1, { name: 'Ethereum', id: 1 }]]),
			error: undefined
		})),
		getAllAccounts: vi.fn(() => ({
			value: new Map(),
			error: undefined
		}))
	})
}));

const mockVault = {
	chainId: 1,
	id: '0x1234567890abcdef1234567890abcdef12345678',
	owner: '0xabcdef1234567890abcdef1234567890abcdef12',
	vaultId: BigInt(42),
	balance: BigInt('1000000000000000000'),
	formattedBalance: '1',
	token: {
		id: '0x1111111111111111111111111111111111111111',
		address: '0x1111111111111111111111111111111111111111',
		name: 'Mock Token',
		symbol: 'MTK',
		decimals: '18'
	},
	orderbook: '0x2222222222222222222222222222222222222222',
	ordersAsInput: [],
	ordersAsOutput: []
} as unknown as RaindexVault;

vi.mock('@tanstack/svelte-query');

// Hoisted mock stores
const {
	mockActiveNetworkRefStore,
	mockActiveOrderbookRefStore,
	mockHideZeroBalanceVaultsStore,
	mockOrderHashStore,
	mockActiveAccountsItemsStore,
	mockShowInactiveOrdersStore,
	mockActiveAccountsStore,
	mockSelectedChainIdsStore,
	mockShowMyItemsOnlyStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

const defaultProps = {
	orderHash: mockOrderHashStore,
	activeAccountsItems: mockActiveAccountsItemsStore,
	showInactiveOrders: mockShowInactiveOrdersStore,
	hideZeroBalanceVaults: mockHideZeroBalanceVaultsStore,
	activeNetworkRef: mockActiveNetworkRefStore,
	activeOrderbookRef: mockActiveOrderbookRefStore,
	activeAccounts: mockActiveAccountsStore,
	selectedChainIds: mockSelectedChainIdsStore,
	showMyItemsOnly: mockShowMyItemsOnlyStore
};

type VaultsListTableProps = ComponentProps<VaultsListTable>;

describe('VaultsListTable', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useAccount as Mock).mockReturnValue({
			matchesAccount: mockMatchesAccount,
			account: mockAccountStore
		});
	});
	it('displays vault information correctly', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVault]] },
					status: 'success',
					isFetching: false,
					isFetched: true
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;
		render(VaultsListTable, defaultProps as unknown as VaultsListTableProps);
		expect(screen.getByTestId('vault-network')).toHaveTextContent('Ethereum');
		expect(screen.getByTestId('vault-token')).toHaveTextContent('Mock Token');
		expect(screen.getByTestId('vault-balance')).toHaveTextContent('1 MTK');
	});

	it('shows deposit/withdraw buttons when handlers are provided', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const handleDepositModal = vi.fn();
		const handleWithdrawModal = vi.fn();

		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVault]] },
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

	it('handles deposit action', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		mockQuery.createInfiniteQuery = vi.fn((__options, _queryClient) => ({
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: { pages: [[mockVault]] },
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

		expect(handleDepositModal).toHaveBeenCalledWith(mockVault, undefined, new Map());
	});

	it('hides action buttons when user is not the vault owner', () => {
		mockMatchesAccount.mockReturnValue(false);
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
