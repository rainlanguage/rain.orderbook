import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from '../lib/components/detail/VaultDetail.svelte';
import { readable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';

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
	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	const queryClient = new QueryClient();

	render(VaultDetail, {
		props: {
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
	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(null);

	const queryClient = new QueryClient();

	render(VaultDetail, {
		props: {
			id: '100',
			network: 'mainnet',
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
		expect(screen.queryByTestId('vaulDetailOrdersAsOutput')).toHaveTextContent('None');
	});
});
