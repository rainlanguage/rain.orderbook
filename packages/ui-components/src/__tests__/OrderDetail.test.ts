import { render, screen, waitFor } from '@testing-library/svelte';
import {
	RaindexClient,
	RaindexOrder,
	RaindexTransaction,
	RaindexVault
} from '@rainlanguage/orderbook';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import OrderDetail from '../lib/components/detail/OrderDetail.svelte';
import { readable, writable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';
import type { ComponentProps } from 'svelte';
import { invalidateTanstackQueries } from '$lib/queries/queryClient';
import { useToasts } from '$lib/providers/toasts/useToasts';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

// Mock the account hook
vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

// Mock the js_api functions
vi.mock('@rainlanguage/orderbook', () => ({
	RaindexClient: vi.fn()
}));

// Mock the query client functions
vi.mock('$lib/queries/queryClient', () => ({
	invalidateTanstackQueries: vi.fn()
}));

const mockErrToast = vi.fn();

vi.mock('$lib/providers/toasts/useToasts', () => ({
	useToasts: vi.fn()
}));

vi.mock('$lib/components/charts/OrderTradesChart.svelte', async () => {
	const mockLightweightCharts = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockLightweightCharts };
});
const orderbookAddress = '0x123456789012345678901234567890123456abcd';
const orderHash = '0x0234';

const defaultProps: ComponentProps<OrderDetail> = {
	chainId: 1,
	orderbookAddress,
	orderHash,
	colorTheme: readable('dark'),
	codeMirrorTheme: readable('dark'),
	lightweightChartsTheme: readable(darkChartTheme),
	onRemove: vi.fn(),
	onDeposit: vi.fn(),
	onWithdraw: vi.fn(),
	onWithdrawAll: vi.fn()
};

const mockVaultsList = () => ({
	items: [],
	getWithdrawableVaults: () => ({ value: [], error: null })
});

const mockOrder: RaindexOrder = {
	chainId: 1,
	orderbook: orderbookAddress,
	id: 'mockId',
	orderBytes: '0x0000000000000000000000000000000000000000...',
	orderHash: orderHash,
	owner: '0x1234567890123456789012345678901234567890',
	active: true,
	meta: undefined,
	rainlang: undefined,
	timestampAdded: BigInt(1234567890),
	inputsList: mockVaultsList(),
	outputsList: mockVaultsList(),
	inputsOutputsList: mockVaultsList(),
	vaultsList: {
		...mockVaultsList(),
		items: [
			{
				chainId: 1,
				vaultType: 'input',
				id: '0x0000000000000000000000000000000000000002',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x0000000000000000000000000000000000000000',
					name: 'MockToken',
					symbol: 'MCK',
					decimals: '18'
				},
				balance: BigInt(0),
				vaultId: BigInt(2),
				owner: '0x1234567890123456789012345678901234567890',
				ordersAsOutput: [],
				ordersAsInput: [],
				orderbook: orderbookAddress
			} as unknown as RaindexVault,
			{
				chainId: 1,
				vaultType: 'output',
				id: '0x0000000000000000000000000000000000000001',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x0000000000000000000000000000000000000000',
					name: 'MockToken2',
					symbol: 'MCK2',
					decimals: '18'
				},
				balance: BigInt(0),
				vaultId: BigInt(1),
				owner: '0x1234567890123456789012345678901234567890',
				ordersAsOutput: [],
				ordersAsInput: [],
				orderbook: orderbookAddress
			} as unknown as RaindexVault
		]
	},
	transaction: {
		blockNumber: BigInt(12345678),
		timestamp: BigInt(1234567890),
		id: '0x0000000000000000000000000000000000000000',
		from: '0x1234567890123456789012345678901234567890'
	} as unknown as RaindexTransaction,
	tradesCount: 0
} as unknown as RaindexOrder;

const mockMatchesAccount = vi.fn();
describe('OrderDetail', () => {
	let queryClient: QueryClient;
	let mockRaindexClient: RaindexClient;

	beforeEach(async () => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		queryClient = new QueryClient();

		(useAccount as Mock).mockReturnValue({
			matchesAccount: mockMatchesAccount
		});

		mockRaindexClient = {
			getOrderByHash: vi.fn().mockResolvedValue({
				value: mockOrder
			})
		} as unknown as RaindexClient;
		(useRaindexClient as Mock).mockReturnValue(mockRaindexClient);

		(useToasts as Mock).mockReturnValue({
			toasts: writable([]),
			errToast: mockErrToast,
			removeToast: vi.fn()
		});
	});

	it('calls the order detail query with the correct order hash', async () => {
		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(mockRaindexClient.getOrderByHash).toHaveBeenCalledWith(1, orderbookAddress, orderHash);
	});

	it('shows the correct empty message when the query returns no data', async () => {
		(mockRaindexClient.getOrderByHash as Mock).mockResolvedValue({ value: null });

		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Order not found')).toBeInTheDocument();
		});
	});

	it('shows the correct data when the query returns data', async () => {
		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Order')).toBeInTheDocument();
			expect(screen.getByText('Orderbook')).toBeInTheDocument();
			expect(screen.getByText('Owner')).toBeInTheDocument();
			expect(screen.getByText('Created')).toBeInTheDocument();
			expect(screen.getByText(orderbookAddress)).toBeInTheDocument();
			expect(screen.getByText('0x1234567890123456789012345678901234567890')).toBeInTheDocument();
		});
	});

	it('shows remove button if owner wallet matches and order is active', async () => {
		mockMatchesAccount.mockReturnValue(true);
		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const removeButton = screen.getByTestId('remove-button');
			expect(removeButton).toBeInTheDocument();
			expect(defaultProps.onRemove).not.toHaveBeenCalled();
		});

		await userEvent.click(screen.getByTestId('remove-button'));

		await waitFor(() => {
			expect(defaultProps.onRemove).toHaveBeenCalledWith(mockRaindexClient, mockOrder);
		});
	});

	it('does not show remove button if account does not match owner', async () => {
		mockMatchesAccount.mockReturnValue(false);

		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.queryByTestId('remove-button')).not.toBeInTheDocument();
		});
	});

	it('does not show remove button if order is not active', async () => {
		// Modify the mock to return an inactive order
		(mockRaindexClient.getOrderByHash as Mock).mockResolvedValue({
			value: {
				...mockOrder,
				active: false
			}
		});

		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.queryByTestId('remove-button')).not.toBeInTheDocument();
		});
	});

	it('refresh button triggers query invalidation when clicked', async () => {
		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(async () => {
			const refreshButton = screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);

			expect(invalidateTanstackQueries).toHaveBeenCalledWith(queryClient, [orderHash]);
		});
	});

	it('failed query invalidation triggers a toast', async () => {
		(invalidateTanstackQueries as Mock).mockRejectedValue(new Error('Failed to refresh'));

		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(async () => {
			const refreshButton = screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);
		});

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledWith('Failed to refresh');
		});
	});

	it('calls onDeposit callback when deposit button is clicked', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const user = userEvent.setup();
		const mockOnDeposit = vi.fn();

		render(OrderDetail, {
			props: {
				...defaultProps,
				onDeposit: mockOnDeposit
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Order')).toBeInTheDocument();

			expect(screen.getByText('Orderbook')).toBeInTheDocument();

			expect(screen.getByText('Owner')).toBeInTheDocument();

			expect(screen.getByText('Created')).toBeInTheDocument();
		});

		const depositButton = screen.getAllByTestId('deposit-button');
		await user.click(depositButton[0]);

		expect(mockOnDeposit).toHaveBeenCalledWith(mockRaindexClient, mockOrder.vaultsList.items[1]);
	});

	it('calls onWithdraw callback when withdraw button is clicked', async () => {
		mockMatchesAccount.mockReturnValue(true);
		const user = userEvent.setup();
		const mockOnWithdraw = vi.fn();

		render(OrderDetail, {
			props: {
				...defaultProps,
				onWithdraw: mockOnWithdraw
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			expect(screen.getByText('Order')).toBeInTheDocument();
			expect(screen.getByText('Orderbook')).toBeInTheDocument();
			expect(screen.getByText('Owner')).toBeInTheDocument();
			expect(screen.getByText('Created')).toBeInTheDocument();
		});

		const withdrawButton = screen.getAllByTestId('withdraw-button');
		await user.click(withdrawButton[0]);

		expect(mockOnWithdraw).toHaveBeenCalledWith(mockRaindexClient, mockOrder.vaultsList.items[1]);
	});
});
