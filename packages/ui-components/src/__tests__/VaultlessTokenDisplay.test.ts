import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import VaultlessTokenDisplay from '../lib/components/VaultlessTokenDisplay.svelte';
import type { RaindexVault } from '@rainlanguage/orderbook';
import { QueryClient } from '@tanstack/svelte-query';

const mockGetOwnerBalance = vi.fn();
const mockGetAllowance = vi.fn();

const mockVault: RaindexVault = {
	id: '0x1234567890abcdef1234567890abcdef12345678',
	chainId: 1,
	orderbook: '0x2222222222222222222222222222222222222222',
	owner: '0x3333333333333333333333333333333333333333',
	token: {
		symbol: 'USDC',
		name: 'USD Coin',
		address: '0x0000000000000000000000000000000000000001',
		decimals: 6
	},
	formattedAmount: '0',
	vaultless: true,
	getOwnerBalance: mockGetOwnerBalance,
	getAllowance: mockGetAllowance
} as unknown as RaindexVault;

const createQueryClient = () =>
	new QueryClient({
		defaultOptions: {
			queries: {
				retry: false,
				gcTime: 0
			}
		}
	});

describe('VaultlessTokenDisplay', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders token name and symbol', async () => {
		mockGetOwnerBalance.mockResolvedValue({
			value: { formattedAmount: '100.00' },
			error: null
		});
		mockGetAllowance.mockResolvedValue({
			value: { formattedAmount: 'Unlimited' },
			error: null
		});

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(screen.getByText(/USD Coin/)).toBeInTheDocument();
		expect(screen.getByText(/USDC/)).toBeInTheDocument();
	});

	it('shows vaultless badge', async () => {
		mockGetOwnerBalance.mockResolvedValue({
			value: { formattedAmount: '100.00' },
			error: null
		});
		mockGetAllowance.mockResolvedValue({
			value: { formattedAmount: 'Unlimited' },
			error: null
		});

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(screen.getByText('Vaultless')).toBeInTheDocument();
	});

	it('displays loading state initially', async () => {
		mockGetOwnerBalance.mockImplementation(
			() =>
				new Promise((resolve) =>
					setTimeout(() => resolve({ value: { formattedAmount: '100' }, error: null }), 100)
				)
		);
		mockGetAllowance.mockImplementation(
			() =>
				new Promise((resolve) =>
					setTimeout(() => resolve({ value: { formattedAmount: '50' }, error: null }), 100)
				)
		);

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(screen.getByText('Loading wallet data...')).toBeInTheDocument();
	});

	it('displays wallet balance and allowance after loading', async () => {
		mockGetOwnerBalance.mockResolvedValue({
			value: { formattedAmount: '100.00' },
			error: null
		});
		mockGetAllowance.mockResolvedValue({
			value: { formattedAmount: 'Unlimited' },
			error: null
		});

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Wallet Balance: 100.00')).toBeInTheDocument();
		});
		expect(screen.getByText('Approved: Unlimited')).toBeInTheDocument();
	});

	it('displays error message when balance fetch fails', async () => {
		mockGetOwnerBalance.mockResolvedValue({
			value: null,
			error: { readableMsg: 'Failed to fetch balance' }
		});
		mockGetAllowance.mockResolvedValue({
			value: { formattedAmount: 'Unlimited' },
			error: null
		});

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText(/Error: Failed to fetch balance/)).toBeInTheDocument();
		});
	});

	it('displays error message when allowance fetch fails', async () => {
		mockGetOwnerBalance.mockResolvedValue({
			value: { formattedAmount: '100.00' },
			error: null
		});
		mockGetAllowance.mockResolvedValue({
			value: null,
			error: { readableMsg: 'Failed to fetch allowance' }
		});

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText(/Error: Failed to fetch allowance/)).toBeInTheDocument();
		});
	});

	it('has correct test id', async () => {
		mockGetOwnerBalance.mockResolvedValue({
			value: { formattedAmount: '100.00' },
			error: null
		});
		mockGetAllowance.mockResolvedValue({
			value: { formattedAmount: 'Unlimited' },
			error: null
		});

		const queryClient = createQueryClient();
		render(VaultlessTokenDisplay, {
			props: { tokenVault: mockVault },
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(screen.getByTestId('vaultless-token-display')).toBeInTheDocument();
	});
});
