import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import OrderDetail from '../lib/components/detail/OrderDetail.svelte';
import { readable, writable } from 'svelte/store';
import { darkChartTheme } from '../lib/utils/lightweightChartsThemes';
import type { Config } from 'wagmi';
import userEvent from '@testing-library/user-event';
import { useAccount } from '$lib/providers/wallet/useAccount';
import { getOrderByHash, type SgOrder } from '@rainlanguage/orderbook/js_api';
import { invalidateIdQuery } from '$lib/queries/queryClient';
import type { ComponentProps } from 'svelte';
import type { Hex } from 'viem';

// Mock the account hook
vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

// Mock the js_api functions
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getOrderByHash: vi.fn()
}));

// Mock the query client functions
vi.mock('$lib/queries/queryClient', () => ({
	invalidateIdQuery: vi.fn()
}));

vi.mock('$lib/components/charts/OrderTradesChart.svelte', async () => {
	const mockLightweightCharts = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockLightweightCharts };
});
const subgraphUrl = 'https://example.com';
const orderbookAddress = '0x123456789012345678901234567890123456abcd';
const chainId = 1;
const rpcUrl = 'https://eth-mainnet.alchemyapi.io/v2/your-api-key';
const orderHash = 'mockOrderHash';

const defaultProps: ComponentProps<OrderDetail> = {
				orderHash,
				rpcUrl,
				subgraphUrl,
				orderbookAddress,
				chainId,
	colorTheme: 'dark',
	codeMirrorTheme: readable('dark'),
	lightweightChartsTheme: readable(darkChartTheme),
	wagmiConfig: writable({} as Config),
	handleOrderRemoveModal: vi.fn(),
	onDeposit: vi.fn(),
	onWithdraw: vi.fn()
};

const mockOrder: SgOrder = {
	id: 'mockId',
	orderBytes: '0x0000000000000000000000000000000000000000...',
	owner: '0x1234567890123456789012345678901234567890',
	orderHash: orderHash,
	active: true,
	meta: null,
	timestampAdded: '1234567890',
	orderbook: { id: orderbookAddress },

	inputs: [
		{
			id: '0x0000000000000000000000000000000000000002',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'MockToken',
				symbol: 'MCK',
				decimals: '18'
			},
			balance: '0',
			vaultId: '0x2',
			owner: '0x1234567890123456789012345678901234567890',
			ordersAsOutput: [],
			ordersAsInput: [],
			balanceChanges: [],
			orderbook: {
				id: orderbookAddress
			}
		}
	],

	outputs: [
		{
			id: '0x0000000000000000000000000000000000000001',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'MockToken2',
				symbol: 'MCK2',
				decimals: '18'
			},
			balance: '0',
			vaultId: '0x1',
			owner: '0x1234567890123456789012345678901234567890',
			ordersAsOutput: [],
			ordersAsInput: [],
			balanceChanges: [],
			orderbook: {
				id: orderbookAddress
			}
		}
	],

	addEvents: [
		{
			transaction: {
				blockNumber: '12345678',
				timestamp: '1234567890',
				id: '0x0000000000000000000000000000000000000000',
				from: '0x1234567890123456789012345678901234567890'
			}
		}
	],
	trades: [],
	removeEvents: [],

	expression: '0x123456' // Your existing field
} as unknown as SgOrder;

const mockAccoutStore = readable('0x1234567890123456789012345678901234567890');

describe('OrderDetail', () => {
	let queryClient: QueryClient;

	beforeEach(async () => {
		vi.clearAllMocks();
		queryClient = new QueryClient();

		// Set up account mock
		(useAccount as Mock).mockReturnValue({
			account: mockAccoutStore
		});

		// Mock getOrderByHash to return our data structure
		(getOrderByHash as Mock).mockResolvedValue({
			order: mockOrder,
			vaults: new Map([
				['inputs', []],
				['outputs', []],
				['inputs_outputs', []]
			])
		});
	});

	it('calls the order detail query with the correct order hash', async () => {
		render(OrderDetail, {
			props: {
				orderHash,
				rpcUrl,
				subgraphUrl,
				orderbookAddress,
				chainId,
				colorTheme: readable('dark'),
				codeMirrorTheme: readable('dark'),
				lightweightChartsTheme: readable(darkChartTheme)
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		expect(getOrderByHash).toHaveBeenCalledWith(subgraphUrl, orderHash);
	});

	it('shows the correct empty message when the query returns no data', async () => {
		(getOrderByHash as Mock).mockResolvedValue(null);

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
			// Check for order hash
			expect(screen.getByText('Order')).toBeInTheDocument();
			// Check for Orderbook field
			expect(screen.getByText('Orderbook')).toBeInTheDocument();
			// Check for Owner field
			expect(screen.getByText('Owner')).toBeInTheDocument();
			// Check for Created field
			expect(screen.getByText('Created')).toBeInTheDocument();
		});
	});

	it('shows remove button if owner wallet matches and order is active', async () => {
		render(OrderDetail, {
			props: defaultProps,
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const removeButton = screen.getByTestId('remove-button');
			expect(removeButton).toBeInTheDocument();
			expect(defaultProps.handleOrderRemoveModal).not.toHaveBeenCalled();
		});

		// Click the Remove button
		await userEvent.click(screen.getByTestId('remove-button'));

		await waitFor(() => {
			expect(defaultProps.handleOrderRemoveModal).toHaveBeenCalledWith({
				open: true,
				args: {
					order: mockOrder,
					onRemove: expect.any(Function),
					chainId,
					orderbookAddress,
					subgraphUrl,
					account: mockAccoutStore
				}
			});
		});
	});

	it('does not show remove button if account does not match owner', async () => {
		(useAccount as Mock).mockReturnValue({
			account: readable('0x0987654321098765432109876543210987654321')
		});

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
		(getOrderByHash as Mock).mockResolvedValue({
			order: {
				...mockOrder,
				active: false
			},
			vaults: new Map([
				['inputs', []],
				['outputs', []],
				['inputs_outputs', []]
			])
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
			const refreshButton = await screen.getByTestId('top-refresh');
			await userEvent.click(refreshButton);

			expect(invalidateIdQuery).toHaveBeenCalledWith(queryClient, orderHash);
		});
	});

	it('calls onDeposit callback when deposit button is clicked', async () => {
		const user = userEvent.setup();
		const mockOnDeposit = vi.fn();

		render(OrderDetail, {
			props: {
				...defaultProps,
				onDeposit: mockOnDeposit,
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		// Wait for the component to finish loading data
		await waitFor(() => {
			expect(screen.queryByText('Order not found')).not.toBeInTheDocument();
		});

		// Find and click the deposit button
		const depositButton = await screen.getByTestId('deposit-button');
		await user.click(depositButton);

		// Verify onDeposit was called with correct parameters
		expect(mockOnDeposit).toHaveBeenCalledWith(mockOrder.inputs[0]);
	});

	it('calls onWithdraw callback when withdraw button is clicked', async () => {
		const user = userEvent.setup();
		const mockOnWithdraw = vi.fn();

		render(OrderDetail, {
			props: {
				...defaultProps,
				onWithdraw: mockOnWithdraw
			},
						context: new Map([['$$_queryClient', queryClient]])

		});

		// Wait for the component to finish loading data
		await waitFor(() => {
			expect(screen.queryByText('Order not found')).not.toBeInTheDocument();
		});

		// Find and click the withdraw button
		const withdrawButton = await screen.getByTestId('withdraw-button');
		await user.click(withdrawButton);

		// Verify onWithdraw was called with correct parameters
		expect(mockOnWithdraw).toHaveBeenCalledWith(mockOrder.inputs[0]);
	});
});
