import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from '../lib/components/detail/VaultDetail.svelte';
import { readable, writable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';
import userEvent from '@testing-library/user-event';
import type { ComponentProps } from 'svelte';

import { getVault, type SgOrderAsIO, type SgVault } from '@rainlanguage/orderbook';
type VaultDetailProps = ComponentProps<VaultDetail>;
import { useAccount } from '../lib/providers/wallet/useAccount';
import { QKEY_VAULT } from '$lib/queries/keys';
import { useToasts } from '../lib/providers/toasts/useToasts';
import { invalidateTanstackQueries } from '$lib/queries/queryClient';

vi.mock('../lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

vi.mock('@rainlanguage/orderbook', () => ({
	getVault: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/services/modal', () => ({
	handleDepositModal: vi.fn(),
	handleWithdrawModal: vi.fn()
}));

vi.mock('$lib/queries/queryClient', () => ({
	invalidateTanstackQueries: vi.fn()
}));

vi.mock('$lib/providers/toasts/useToasts', () => ({
	useToasts: vi.fn()
}));

const mockErrToast = vi.fn();

const mockSettings = readable({
	orderbook: {
		subgraphs: {
			mainnet: {
				key: 'mainnet',
				url: 'https://example.com'
			}
		}
	}
});

const defaultProps: VaultDetailProps = {
	id: '100',
	network: 'mainnet',
	activeNetworkRef: writable('mainnet'),
	activeOrderbookRef: writable('0x00'),
	settings: mockSettings,
	lightweightChartsTheme: readable(darkChartTheme),
	onDeposit: vi.fn(),
	onWithdraw: vi.fn()
} as unknown as VaultDetailProps;

const mockMatchesAccount = vi.fn();

describe('VaultDetail', () => {
	let queryClient: QueryClient;
	let mockData: SgVault;

	beforeEach(async () => {
		vi.clearAllMocks();
		queryClient = new QueryClient();

		mockMatchesAccount.mockReturnValue(true);

		(useAccount as Mock).mockReturnValue({
			matchesAccount: mockMatchesAccount
		});

		(useToasts as Mock).mockReturnValue({
			errToast: mockErrToast,
			toasts: writable([]),
			removeToast: vi.fn()
		});

		mockData = {
			id: '1',
			vaultId: '0xabc',
			owner: '0x1234567890123456789012345678901234567890',
			token: {
				id: '0x456',
				address: '0x456',
				name: 'USDC coin',
				symbol: 'USDC',
				decimals: '6'
			},
			balance: '100000000000',
			ordersAsInput: [],
			ordersAsOutput: [],
			balanceChanges: [],
			orderbook: {
				id: '0x00'
			}
		} as unknown as SgVault;

		(getVault as Mock).mockResolvedValue({ value: mockData });
	});

	it('calls the vault detail query fn with the correct vault id', async () => {
		const { getVault } = await import('@rainlanguage/orderbook');

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(getVault).toHaveBeenCalledWith('https://example.com', '100');
	});

	it('shows the correct empty message when the query returns no data', async () => {
		const { getVault } = await import('@rainlanguage/orderbook');
		(getVault as Mock).mockResolvedValue({ value: null });

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Vault not found')).toBeInTheDocument();
		});
	});

	it('shows the correct data when the query returns data', async () => {
		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('vaultDetailTokenName')).toHaveTextContent('USDC coin');
			expect(screen.getByTestId('vaultDetailVaultId')).toHaveTextContent('Vault ID 0xabc');
			expect(screen.getByTestId('vaultDetailOwnerAddress')).toHaveTextContent(
				'Owner address 0x123'
			);
			expect(screen.getByTestId('vaultDetailTokenAddress')).toHaveTextContent(
				'Token address 0x456'
			);
			expect(screen.getByTestId('vaultDetailBalance')).toHaveTextContent('Balance 100000 USDC');
			expect(screen.queryByTestId('vaultDetailOrdersAsInput')).toHaveTextContent('None');
			expect(screen.queryByTestId('vaultDetailOrdersAsOutput')).toHaveTextContent('None');
		});
	});

	it('shows deposit/withdraw buttons when account matches are met', async () => {
		mockMatchesAccount.mockReturnValue(true);

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getAllByTestId('deposit-button')).toHaveLength(1);
			expect(screen.getAllByTestId('withdraw-button')).toHaveLength(1);
		});
	});

	it("doesn't show deposit/withdraw buttons when account doesn't match", async () => {
		mockMatchesAccount.mockReturnValue(false);

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.queryByTestId('depositOrWithdrawButton')).not.toBeInTheDocument();
		});
	});

	it('refresh button triggers query invalidation when clicked', async () => {
		mockData.ordersAsInput = [{ id: '1', owner: '0x123' }] as unknown as SgOrderAsIO[];
		mockData.ordersAsOutput = [{ id: '2', owner: '0x123' }] as unknown as SgOrderAsIO[];

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(async () => {
			const refreshButton = await screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);
			expect(invalidateTanstackQueries).toHaveBeenCalledWith(queryClient, [
				'100',
				QKEY_VAULT + '100'
			]);
		});
	});

	it('failed query invalidation triggers a toast', async () => {
		mockData.ordersAsInput = [{ id: '1', owner: '0x123' }] as unknown as SgOrderAsIO[];
		mockData.ordersAsOutput = [{ id: '2', owner: '0x123' }] as unknown as SgOrderAsIO[];

		(invalidateTanstackQueries as Mock).mockRejectedValue(new Error('Failed to refresh'));

		render(VaultDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(async () => {
			const refreshButton = await screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);
			expect(mockErrToast).toHaveBeenCalledWith('Failed to refresh');
		});
	});
});
