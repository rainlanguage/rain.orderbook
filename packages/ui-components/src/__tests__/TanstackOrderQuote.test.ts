import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, type Mock } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import TanstackOrderQuote from '../lib/components/detail/TanstackOrderQuote.svelte';
import { expect } from '../lib/test/matchers';
import { mockOrderDetailsExtended } from '../lib/__fixtures__/orderDetail';
import { getOrderQuote } from '@rainlanguage/orderbook';
import type { SgOrder } from '@rainlanguage/orderbook';
import { useToasts } from '$lib/providers/toasts/useToasts';
import { writable } from 'svelte/store';
import { invalidateTanstackQueries } from '$lib/queries/queryClient';

vi.mock('@rainlanguage/orderbook', () => ({
	getOrderQuote: vi.fn()
}));

vi.mock('$lib/providers/toasts/useToasts', () => ({
	useToasts: vi.fn()
}));

vi.mock('$lib/queries/queryClient', async (importOriginal) => ({
	...((await importOriginal()) as object),
	invalidateTanstackQueries: vi.fn()
}));

const mockErrToast = vi.fn();

describe('TanstackOrderQuote component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		(useToasts as Mock).mockReturnValue({
			toasts: writable([]),
			errToast: mockErrToast,
			removeToast: vi.fn()
		});
	});
	it('displays order quote data when query is successful', async () => {
		(getOrderQuote as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' },
					error: undefined
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				id: '0x123',
				order: mockOrderDetailsExtended.order as SgOrder,
				rpcUrls: ['https://example.com'],
				orderbookAddress: '0x123',
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const orderQuoteComponent = screen.getByTestId('bodyRow');

			expect(orderQuoteComponent).toHaveTextContent('ETH/USDT');
			expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692');
			expect(orderQuoteComponent).toHaveTextContent('6.563567234157974775');
		});
	});

	it('refreshes the quote when the refresh icon is clicked', async () => {
		(getOrderQuote as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' },
					error: undefined
				},
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'BTC/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x54fa82f5c7001dad', ratio: '0x53e0089714d06709' },
					error: undefined
				}
			]
		});
		// Setup mock for the second data load (after refresh)
		(getOrderQuote as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x5282713eceeccb5e', ratio: '0x577fe09a8775137c' },
					error: undefined
				},
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'BTC/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x5430775053da5e53', ratio: '0x5a01719c871bb83f' },
					error: undefined
				}
			]
		});

		// Mock the invalidateTanstackQueries function to ensure it works correctly
		(invalidateTanstackQueries as Mock).mockImplementation((queryClient, queryKey) => {
			// This will trigger a refetch which will use the second mock
			queryClient.invalidateQueries({ queryKey });
			return Promise.resolve();
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				id: '0x123',
				order: mockOrderDetailsExtended.order as SgOrder,
				rpcUrls: ['https://example.com'],
				orderbookAddress: '0x123',
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const orderQuoteRows = screen.getAllByTestId('bodyRow');

			// Check ETH/USDT row
			expect(orderQuoteRows[0]).toHaveTextContent('1.550122181502135692');

			// Check BTC/USDT row
			expect(orderQuoteRows[1]).toHaveTextContent('6.123350635480882605');
		});

		const refreshButton = screen.getByTestId('refresh-button');
		fireEvent.click(refreshButton);

		await waitFor(() => {
			const orderQuoteRows = screen.getAllByTestId('bodyRow');

			// Check ETH/USD row
			expect(orderQuoteRows[0]).toHaveTextContent('ETH/USD');
			expect(orderQuoteRows[0]).toHaveTextContent('5.945438972656012126');
			expect(orderQuoteRows[0]).toHaveTextContent('6.305004957644166012');

			// Check BTC/USDT row
			expect(orderQuoteRows[1]).toHaveTextContent('BTC/USDT');
			expect(orderQuoteRows[1]).toHaveTextContent('6.066479884955967059');
			expect(orderQuoteRows[1]).toHaveTextContent('6.485589855485802559');
		});
	});

	it('displays error message when query fails', async () => {
		(getOrderQuote as Mock).mockResolvedValueOnce({
			value: [
				{
					success: false,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: undefined,
					error: 'Network error'
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				id: '0x123',
				order: mockOrderDetailsExtended.order as SgOrder,
				rpcUrls: ['https://example.com'],
				orderbookAddress: '0x123',
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const errorCell = screen.getByText((content) => content.includes('Error fetching quote'));
			expect(errorCell).toBeInTheDocument();
		});
	});

	it('displays zero for price when io ratio is zero', async () => {
		(getOrderQuote as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x158323e942e36d8c', ratio: '0x0' },
					error: undefined
				}
			]
		});

		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				id: '0x123',
				order: mockOrderDetailsExtended.order as SgOrder,
				rpcUrls: ['https://example.com'],
				orderbookAddress: '0x123',
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		await waitFor(() => {
			const orderQuoteComponent = screen.getByTestId('bodyRow');

			expect(orderQuoteComponent).toHaveTextContent('ETH/USDT');
			expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692'); // maxOutput
			expect(orderQuoteComponent).toHaveTextContent('0'); // ratio
			expect(orderQuoteComponent).toHaveTextContent('(0)'); // inverse price
		});
	});

	it('triggers a toast when the quote fails to refresh', async () => {
		(invalidateTanstackQueries as Mock).mockRejectedValueOnce(new Error('Failed to refresh'));
		(getOrderQuote as Mock).mockResolvedValueOnce({
			value: [
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'ETH/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' },
					error: undefined
				},
				{
					success: true,
					block_number: '0x123',
					pair: { pairName: 'BTC/USDT', inputIndex: 0, outputIndex: 1 },
					data: { maxOutput: '0x54fa82f5c7001dad', ratio: '0x53e0089714d06709' },
					error: undefined
				}
			]
		});
		const queryClient = new QueryClient();

		render(TanstackOrderQuote, {
			props: {
				id: '0x123',
				order: mockOrderDetailsExtended.order as SgOrder,
				rpcUrls: ['https://example.com'],
				orderbookAddress: '0x123',
				handleQuoteDebugModal: vi.fn()
			},
			context: new Map([['$$_queryClient', queryClient]])
		});

		const refreshButton = screen.getByTestId('refresh-button');
		fireEvent.click(refreshButton);

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledWith('Failed to refresh');
		});
	});
});
