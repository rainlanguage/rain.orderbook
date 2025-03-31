import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from '../lib/components/detail/VaultDetail.svelte';
import { readable, writable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';
import type { Config } from 'wagmi';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

// Mock the js_api getVault function
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getVault: vi.fn()
}));

// Mock navigation
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

// Mock modal handlers
vi.mock('$lib/services/modal', () => ({
	handleDepositModal: vi.fn(),
	handleWithdrawModal: vi.fn()
}));

const mockSettings = readable({
	subgraphs: {
		mainnet: 'https://example.com'
	}
});

test('calls the vault detail query fn with the correct vault id', async () => {
	(useAccount as Mock).mockReturnValue({
		account: writable('0x123')
	});
	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	const queryClient = new QueryClient();

	render(VaultDetail, {
		props: {
			activeNetworkRef: writable('mainnet'),
			activeOrderbookRef: writable('0x00'),
			id: '100',
			network: 'mainnet',
			settings: mockSettings,
			lightweightChartsTheme: readable(darkChartTheme)
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	expect(getVault).toHaveBeenCalledWith('https://example.com', '100');
});

test('shows the correct empty message when the query returns no data', async () => {
	(useAccount as Mock).mockReturnValue({
		account: writable('0x123')
	});
	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(null);

	const queryClient = new QueryClient();

	render(VaultDetail, {
		props: {
			id: '100',
			network: 'mainnet',
			activeNetworkRef: writable('mainnet'),
			activeOrderbookRef: writable('0x00'),
			settings: mockSettings,
			lightweightChartsTheme: readable(darkChartTheme)
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(screen.getByText('Vault not found')).toBeInTheDocument();
	});
});

test('shows the correct data when the query returns data', async () => {
	(useAccount as Mock).mockReturnValue({
		account: writable('0x123')
	});
	const mockData = {
		id: '1',
		vaultId: '0xabc',
		owner: '0x123',
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
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);

	const queryClient = new QueryClient();

	render(VaultDetail, {
		props: {
			id: '100',
			network: 'mainnet',
			activeNetworkRef: writable('mainnet'),
			activeOrderbookRef: writable('0x00'),
			settings: mockSettings,
			lightweightChartsTheme: readable(darkChartTheme)
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(screen.getByTestId('vaultDetailTokenName')).toHaveTextContent('USDC coin');
		expect(screen.getByTestId('vaultDetailVaultId')).toHaveTextContent('Vault ID 0xabc');
		expect(screen.getByTestId('vaultDetailOwnerAddress')).toHaveTextContent('Owner Address 0x123');
		expect(screen.getByTestId('vaultDetailTokenAddress')).toHaveTextContent('Token address 0x456');
		expect(screen.getByTestId('vaultDetailBalance')).toHaveTextContent('Balance 100000 USDC');
		expect(screen.queryByTestId('vaultDetailOrdersAsInput')).toHaveTextContent('None');
		expect(screen.queryByTestId('vaultDetailOrdersAsOutput')).toHaveTextContent('None');
	});
});

test('shows deposit/withdraw buttons when account address matches owner', async () => {
	const mockData = {
		id: '1',
		vaultId: '0xabc',
		owner: '0x123',
		token: {
			id: '0x456',
			address: '0x456',
			name: 'USDC coin',
			symbol: 'USDC',
			decimals: '6'
		},
		balance: '100000000000',
		ordersAsInput: [
			{
				id: '1',
				owner: '0x123'
			}
		],
		ordersAsOutput: [
			{
				id: '2',
				owner: '0x123'
			}
		],
		balanceChanges: [],
		orderbook: {
			id: '0x00'
		}
	};

	(useAccount as Mock).mockReturnValue({
		account: writable('0x123')
	});

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);

	const queryClient = new QueryClient();
	const mockWagmiConfig = writable({} as Config);

	render(VaultDetail, {
		props: {
			id: '100',
			network: 'mainnet',
			activeNetworkRef: writable('mainnet'),
			activeOrderbookRef: writable('0x00'),
			settings: mockSettings,
			lightweightChartsTheme: readable(darkChartTheme),
			wagmiConfig: mockWagmiConfig,
			handleDepositOrWithdrawModal: vi.fn()
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(screen.getAllByTestId('depositOrWithdrawButton')).toHaveLength(2);
	});
});

test('refresh button triggers query invalidation when clicked', async () => {
	const mockData = {
		id: '1',
		vaultId: '0xabc',
		owner: '0x123',
		token: {
			id: '0x456',
			address: '0x456',
			name: 'USDC coin',
			symbol: 'USDC',
			decimals: '6'
		},
		balance: '100000000000',
		ordersAsInput: [
			{
				id: '1',
				owner: '0x123'
			}
		],
		ordersAsOutput: [
			{
				id: '2',
				owner: '0x123'
			}
		],
		balanceChanges: [],
		orderbook: {
			id: '0x00'
		}
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);
	const queryClient = new QueryClient();
	const invalidateQueries = vi.spyOn(queryClient, 'invalidateQueries');

	const mockWagmiConfig = writable({} as Config);

	render(VaultDetail, {
		props: {
			id: '100',
			network: 'mainnet',
			activeNetworkRef: writable('mainnet'),
			activeOrderbookRef: writable('0x00'),
			settings: mockSettings,
			lightweightChartsTheme: readable(darkChartTheme),
			wagmiConfig: mockWagmiConfig,
			handleDepositOrWithdrawModal: vi.fn()
		},
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(async () => {
		const refreshButton = await screen.findAllByTestId('refresh-button');
		await userEvent.click(refreshButton[0]);
		expect(invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['100'],
			refetchType: 'all',
			exact: false
		});
	});
});
