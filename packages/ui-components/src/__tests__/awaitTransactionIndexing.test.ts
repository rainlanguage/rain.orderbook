import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { SgAddOrderWithOrder, SgTransaction } from '@rainlanguage/orderbook';
import { awaitSubgraphIndexing, TIMEOUT_ERROR } from '$lib/services/awaitTransactionIndexing';

vi.mock('@rainlanguage/orderbook', () => ({
	getTransaction: vi.fn(),
	getTransactionAddOrders: vi.fn(),
	getTransactionRemoveOrders: vi.fn()
}));

describe('subgraphIndexing', () => {
	const mockFetchData = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.clearAllMocks();
	});

	it('should resolve with value when data is successfully fetched', async () => {
		const mockData = { id: 'tx123' } as SgTransaction;
		mockFetchData.mockResolvedValue({ value: mockData });

		const resultPromise = awaitSubgraphIndexing({
			subgraphUrl: 'https://test.subgraph.com',
			txHash: 'tx123',
			successMessage: 'Transaction confirmed',
			fetchEntityFn: mockFetchData,
			isSuccess: (data: SgTransaction) => !!data.id
		});

		await vi.advanceTimersByTimeAsync(1000);

		const result = await resultPromise;

		expect(result.value).toBeDefined();
		expect(result.error).toBeUndefined();
		expect(result.value?.txHash).toBe('tx123');
		expect(result.value?.successMessage).toBe('Transaction confirmed');
		expect(result.value?.data).toEqual(mockData);

		expect(mockFetchData).toHaveBeenCalledWith('https://test.subgraph.com', 'tx123');
		expect(mockFetchData).toHaveBeenCalledTimes(1);
	});

	it('should extract order hash from array data', async () => {
		const mockOrderData = [
			{
				order: {
					orderHash: 'order123'
				}
			}
		] as SgAddOrderWithOrder[];

		mockFetchData.mockResolvedValue({ value: mockOrderData });

		const resultPromise = awaitSubgraphIndexing({
			subgraphUrl: 'https://test.subgraph.com',
			txHash: 'tx123',
			successMessage: 'Order confirmed',
			network: 'mainnet',
			fetchEntityFn: mockFetchData,
			isSuccess: (data: SgAddOrderWithOrder[]) => data.length > 0
		});

		await vi.advanceTimersByTimeAsync(1000);

		const result = await resultPromise;

		expect(result.value).toBeDefined();
		expect(result.value?.orderHash).toBe('order123');
		expect(result.value?.network).toBe('mainnet');
	});

	it('should retry fetching data until maxAttempts is reached', async () => {
		mockFetchData.mockResolvedValue({ value: null });

		const resultPromise = awaitSubgraphIndexing({
			subgraphUrl: 'https://test.subgraph.com',
			txHash: 'tx123',
			successMessage: 'Transaction confirmed',
			maxAttempts: 5,
			interval: 500,
			fetchEntityFn: mockFetchData,
			isSuccess: () => false
		});

		for (let i = 0; i < 5; i++) {
			await vi.advanceTimersByTimeAsync(500);
		}

		const result = await resultPromise;

		expect(result.error).toBeDefined();
		expect(result.value).toBeUndefined();
		expect(result.error).toBe(TIMEOUT_ERROR);
		expect(mockFetchData).toHaveBeenCalledTimes(5);
	});

	it('should handle fetch errors gracefully', async () => {
		mockFetchData.mockResolvedValue({ error: 'error' });

		const resultPromise = awaitSubgraphIndexing({
			subgraphUrl: 'https://test.subgraph.com',
			txHash: 'tx123',
			successMessage: 'Transaction confirmed',
			maxAttempts: 3,
			interval: 500,
			fetchEntityFn: mockFetchData,
			isSuccess: () => true
		});

		for (let i = 0; i < 3; i++) {
			await vi.advanceTimersByTimeAsync(500);
		}

		const result = await resultPromise;

		expect(result.error).toBeDefined();
		expect(result.value).toBeUndefined();
		expect(result.error).toBe(TIMEOUT_ERROR);
	});

	it('should resolve immediately when successful data is found', async () => {
		mockFetchData
			.mockResolvedValueOnce({ value: null })
			.mockResolvedValueOnce({ value: { id: 'tx123' } as SgTransaction });

		const resultPromise = awaitSubgraphIndexing({
			subgraphUrl: 'https://test.subgraph.com',
			txHash: 'tx123',
			successMessage: 'Transaction confirmed',
			maxAttempts: 5,
			interval: 500,
			fetchEntityFn: mockFetchData,
			isSuccess: (data: SgTransaction) => !!data?.id
		});

		await vi.advanceTimersByTimeAsync(500);
		await vi.advanceTimersByTimeAsync(500);

		const result = await resultPromise;

		expect(result.value).toBeDefined();
		expect(result.error).toBeUndefined();
		expect(mockFetchData).toHaveBeenCalledTimes(2);
	});
});
