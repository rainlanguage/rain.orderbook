import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, beforeEach } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from '../lib/components/detail/VaultDetail.svelte';
import { readable, writable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';
import userEvent from '@testing-library/user-event';
import type { ComponentProps } from 'svelte';

type VaultDetailProps = ComponentProps<VaultDetail>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getVault: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/services/modal', () => ({
	handleDepositModal: vi.fn(),
	handleWithdrawModal: vi.fn()
}));

const mockSettings = readable({
	subgraphs: {
		mainnet: 'https://example.com'
	}
});

let queryClient: QueryClient;
let defaultProps: VaultDetailProps;

beforeEach(() => {
	vi.resetAllMocks();
	queryClient = new QueryClient();
	defaultProps = {
		id: '100',
		network: 'mainnet',
		activeNetworkRef: writable('mainnet'),
		activeOrderbookRef: writable('0x00'),
		settings: mockSettings,
		lightweightChartsTheme: readable(darkChartTheme),
		onDeposit: vi.fn(),
		onWithdraw: vi.fn(),
		signerAddress: writable('0x1234567890123456789012345678901234567890')
	};
});

test('calls the vault detail query fn with the correct vault id', async () => {
	const { getVault } = await import('@rainlanguage/orderbook/js_api');

	render(VaultDetail, {
		props: defaultProps,
		context: new Map([['$$_queryClient', queryClient]])
	});

	expect(getVault).toHaveBeenCalledWith('https://example.com', '100');
});

test('shows the correct empty message when the query returns no data', async () => {
	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(null);

	render(VaultDetail, {
		props: defaultProps,
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
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);

	render(VaultDetail, {
		props: defaultProps,
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

test('shows deposit/withdraw buttons when signerAddress matches owner', async () => {
	const mockData = {
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
		ordersAsInput: [
			{
				id: '1',
				owner: '0x1234567890123456789012345678901234567890'
			}
		],
		ordersAsOutput: [
			{
				id: '2',
				owner: '0x1234567890123456789012345678901234567890'
			}
		],
		balanceChanges: [],
		orderbook: {
			id: '0x00'
		}
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);

	const propsWithSigner = {
		...defaultProps,
		signerAddress: writable('0x1234567890123456789012345678901234567890')
	};

	render(VaultDetail, {
		props: propsWithSigner,
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(screen.getByTestId('deposit-button')).toBeInTheDocument();
		expect(screen.getByTestId('withdraw-button')).toBeInTheDocument();
	});
});

test('does not show deposit/withdraw buttons when signerAddress does not match owner', async () => {
	const mockData = {
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
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);


	const propsWithNonMatchingSigner = {
		...defaultProps,
		signerAddress: writable('0x9876543210987654321098765432109876543210')
	};

	render(VaultDetail, {
		props: propsWithNonMatchingSigner,
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(() => {
		expect(screen.queryByTestId('deposit-button')).not.toBeInTheDocument();
		expect(screen.queryByTestId('withdraw-button')).not.toBeInTheDocument();
	});
});

test('calls onDeposit callback when deposit button is clicked', async () => {
	const mockData = {
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
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);

	const mockOnDeposit = vi.fn();

	const propsWithCallbacks = {
		...defaultProps,
		onDeposit: mockOnDeposit
	};

	render(VaultDetail, {
		props: propsWithCallbacks,
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(async () => {
		const depositButton = await screen.findByTestId('deposit-button');
		await userEvent.click(depositButton);

		expect(mockOnDeposit).toHaveBeenCalledWith(mockData);
	});
});

test('calls onWithdraw callback when withdraw button is clicked', async () => {
	const mockData = {
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
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);

	const mockOnWithdraw = vi.fn();

	const propsWithCallbacks = {
		...defaultProps,
		onWithdraw: mockOnWithdraw
	};

	render(VaultDetail, {
		props: propsWithCallbacks,
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(async () => {
		const withdrawButton = await screen.findByTestId('withdraw-button');
		await userEvent.click(withdrawButton);

		expect(mockOnWithdraw).toHaveBeenCalledWith(mockData);
	});
});

test('refresh button triggers query invalidation when clicked', async () => {
	const mockData = {
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
	};

	const { getVault } = await import('@rainlanguage/orderbook/js_api');
	vi.mocked(getVault).mockResolvedValue(mockData);
	const invalidateQueries = vi.spyOn(queryClient, 'invalidateQueries');

	const propsWithSigner = {
		...defaultProps,
	};

	render(VaultDetail, {
		props: propsWithSigner,
		context: new Map([['$$_queryClient', queryClient]])
	});

	await waitFor(async () => {
		const refreshButton = await screen.findByTestId('refresh-button');
		await userEvent.click(refreshButton);

		expect(invalidateQueries).toHaveBeenCalledWith(
			expect.objectContaining({
				queryKey: ['100'],
				exact: false
			})
		);
	});
});
