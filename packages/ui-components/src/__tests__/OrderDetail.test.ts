import { render, screen, waitFor } from '@testing-library/svelte';
import { getOrderByHash, type SgOrder } from '@rainlanguage/orderbook';
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
// Mock the account hook
vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

// Mock the js_api functions
vi.mock('@rainlanguage/orderbook', () => ({
	getOrderByHash: vi.fn()
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
const subgraphUrl = 'https://example.com';
const orderbookAddress = '0x123456789012345678901234567890123456abcd';
const rpcUrl = 'https://eth-mainnet.alchemyapi.io/v2/your-api-key';
const orderHash = 'mockOrderHash';

const defaultProps: ComponentProps<OrderDetail> = {
	orderHash,
	rpcUrl,
	subgraphUrl,
	orderbookAddress,
	colorTheme: readable('dark'),
	codeMirrorTheme: readable('dark'),
	lightweightChartsTheme: readable(darkChartTheme),
	onRemove: vi.fn(),
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

const mockMatchesAccount = vi.fn();
describe('OrderDetail', () => {
	let queryClient: QueryClient;

	beforeEach(async () => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		queryClient = new QueryClient();

		(useAccount as Mock).mockReturnValue({
			matchesAccount: mockMatchesAccount
		});

		(getOrderByHash as Mock).mockResolvedValue({
			value: {
				order: mockOrder,
				vaults: new Map([
					['inputs', [mockOrder.inputs[0]]],
					['outputs', [mockOrder.outputs[0]]],
					['inputs_outputs', []]
				])
			}
		});

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

		expect(getOrderByHash).toHaveBeenCalledWith(subgraphUrl, orderHash);
	});

	it('shows the correct empty message when the query returns no data', async () => {
		(getOrderByHash as Mock).mockResolvedValue({ value: null });

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
			expect(defaultProps.onRemove).toHaveBeenCalledWith(mockOrder);
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
		(getOrderByHash as Mock).mockResolvedValue({
			value: {
				order: {
					...mockOrder,
					active: false
				},
				vaults: new Map([
					['inputs', []],
					['outputs', []],
					['inputs_outputs', []]
				])
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
			const refreshButton = await screen.getByTestId('top-refresh');
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
			const refreshButton = await screen.getByTestId('top-refresh');
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

		const depositButton = await screen.getAllByTestId('deposit-button');
		await user.click(depositButton[0]);

		expect(mockOnDeposit).toHaveBeenCalledWith(mockOrder.outputs[0]);
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

		const withdrawButton = await screen.getAllByTestId('withdraw-button');
		await user.click(withdrawButton[0]);

		expect(mockOnWithdraw).toHaveBeenCalledWith(mockOrder.outputs[0]);
	});
});
