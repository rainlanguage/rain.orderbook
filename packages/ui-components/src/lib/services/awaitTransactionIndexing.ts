// subgraphIndexing.ts
import type {
	SgAddOrderWithOrder,
	SgRemoveOrderWithOrder,
	SgTransaction
} from '@rainlanguage/orderbook';
import {
	getTransaction,
	getTransactionAddOrders,
	getTransactionRemoveOrders
} from '@rainlanguage/orderbook';

export const TIMEOUT_ERROR =
	'The subgraph took too long to respond. Your transaction may still be processing.';

export type IndexingResult<T> = {
	value?: {
		txHash: string;
		successMessage: string;
		orderHash?: string;
		network?: string;
		data?: T;
	};
	error?: string;
};

/**
 * Generic function to handle waiting for subgraph indexing
 * Returns a promise that resolves with an object containing either value or error
 */
export const awaitSubgraphIndexing = async <T>(options: {
	subgraphUrl: string;
	txHash: string;
	successMessage: string;
	maxAttempts?: number;
	interval?: number;
	network?: string;
	fetchData: (subgraphUrl: string, txHash: string) => Promise<T | null | undefined>;
	isSuccess: (data: T) => boolean;
}): Promise<IndexingResult<T>> => {
	const {
		subgraphUrl,
		txHash,
		successMessage,
		maxAttempts = 10,
		interval = 1000,
		network,
		fetchData,
		isSuccess
	} = options;

	return new Promise((resolve) => {
		let attempts = 0;

		const checkInterval = setInterval(async () => {
			attempts++;
			try {
				const data = await fetchData(subgraphUrl, txHash);

				if (data && isSuccess(data)) {
					clearInterval(checkInterval);

					let orderHash;
					if (Array.isArray(data) && data.length > 0 && data[0]?.order?.orderHash) {
						orderHash = data[0].order.orderHash;
					}

					resolve({
						value: {
							txHash,
							successMessage,
							orderHash,
							network,
							data
						}
					});

					return;
				}
			} catch {
				// Continue with the next attempt
			}

			if (attempts >= maxAttempts) {
				clearInterval(checkInterval);
				resolve({
					error: TIMEOUT_ERROR
				});
				return;
			}
		}, interval);
	});
};

// Helper functions for common scenarios
export const getTransactionConfig = (
	subgraphUrl: string,
	txHash: string,
	successMessage: string,
	network?: string
) => ({
	subgraphUrl,
	txHash,
	successMessage,
	network,
	fetchData: getTransaction,
	isSuccess: (tx: SgTransaction) => !!tx
});

export const getNewOrderConfig = (
	subgraphUrl: string,
	txHash: string,
	successMessage: string,
	network?: string
) => ({
	subgraphUrl,
	txHash,
	successMessage,
	network,
	fetchData: getTransactionAddOrders,
	isSuccess: (addOrders: SgAddOrderWithOrder[]) => addOrders?.length > 0
});

export const getRemoveOrderConfig = (
	subgraphUrl: string,
	txHash: string,
	successMessage: string
) => ({
	subgraphUrl,
	txHash,
	successMessage,
	fetchData: getTransactionRemoveOrders,
	isSuccess: (removeOrders: SgRemoveOrderWithOrder[]) => removeOrders?.length > 0
});

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach, afterEach } = import.meta.vitest;
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
		});

		it('should resolve with value when data is successfully fetched', async () => {
			const mockData = { id: 'tx123' };
			mockFetchData.mockResolvedValue(mockData);

			const resultPromise = awaitSubgraphIndexing({
				subgraphUrl: 'https://test.subgraph.com',
				txHash: 'tx123',
				successMessage: 'Transaction confirmed',
				fetchData: mockFetchData,
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				isSuccess: (data: any) => !!data.id
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
			];

			mockFetchData.mockResolvedValue(mockOrderData);

			const resultPromise = awaitSubgraphIndexing({
				subgraphUrl: 'https://test.subgraph.com',
				txHash: 'tx123',
				successMessage: 'Order confirmed',
				network: 'mainnet',
				fetchData: mockFetchData,

				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				isSuccess: (data: any) => data.length > 0
			});

			await vi.advanceTimersByTimeAsync(1000);

			const result = await resultPromise;

			expect(result.value).toBeDefined();
			expect(result.value?.orderHash).toBe('order123');
			expect(result.value?.network).toBe('mainnet');
		});

		it('should retry fetching data until maxAttempts is reached', async () => {
			mockFetchData.mockResolvedValue(null);

			const resultPromise = awaitSubgraphIndexing({
				subgraphUrl: 'https://test.subgraph.com',
				txHash: 'tx123',
				successMessage: 'Transaction confirmed',
				maxAttempts: 5,
				interval: 500,
				fetchData: mockFetchData,
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
			mockFetchData.mockRejectedValue(new Error('Network error'));

			const resultPromise = awaitSubgraphIndexing({
				subgraphUrl: 'https://test.subgraph.com',
				txHash: 'tx123',
				successMessage: 'Transaction confirmed',
				maxAttempts: 3,
				interval: 500,
				fetchData: mockFetchData,
				isSuccess: () => true
			});

			for (let i = 0; i < 3; i++) {
				await vi.advanceTimersByTimeAsync(500);
			}

			const result = await resultPromise;

			expect(result.error).toBeDefined();
			expect(result.value).toBeUndefined();
			expect(result.error).toBe(TIMEOUT_ERROR);
			expect(mockFetchData).toHaveBeenCalledTimes(3);
		});

		it('should resolve immediately when successful data is found', async () => {
			mockFetchData.mockResolvedValueOnce(null).mockResolvedValueOnce({ id: 'tx123' });

			const resultPromise = awaitSubgraphIndexing({
				subgraphUrl: 'https://test.subgraph.com',
				txHash: 'tx123',
				successMessage: 'Transaction confirmed',
				maxAttempts: 5,
				interval: 500,
				fetchData: mockFetchData,
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				isSuccess: (data: any) => !!data?.id
			});

			await vi.advanceTimersByTimeAsync(500);
			await vi.advanceTimersByTimeAsync(500);

			const result = await resultPromise;

			expect(result.value).toBeDefined();
			expect(result.error).toBeUndefined();
			expect(mockFetchData).toHaveBeenCalledTimes(2);
		});
	});

	describe('helper functions', () => {
		it('getTransactionConfig should return correct configuration', () => {
			const config = getTransactionConfig(
				'https://test.subgraph.com',
				'tx123',
				'Transaction confirmed',
				'mainnet'
			);

			expect(config.subgraphUrl).toBe('https://test.subgraph.com');
			expect(config.txHash).toBe('tx123');
			expect(config.successMessage).toBe('Transaction confirmed');
			expect(config.network).toBe('mainnet');
			expect(config.fetchData).toBe(getTransaction);

			expect(config.isSuccess({ id: 'tx123' } as SgTransaction)).toBe(true);
			expect(config.isSuccess(null as unknown as SgTransaction)).toBe(false);
		});

		it('getNewOrderConfig should return correct configuration', () => {
			const config = getNewOrderConfig(
				'https://test.subgraph.com',
				'tx123',
				'Order added',
				'testnet'
			);

			expect(config.subgraphUrl).toBe('https://test.subgraph.com');
			expect(config.txHash).toBe('tx123');
			expect(config.successMessage).toBe('Order added');
			expect(config.network).toBe('testnet');
			expect(config.fetchData).toBe(getTransactionAddOrders);

			expect(config.isSuccess([{ order: { id: 'order1' } } as SgAddOrderWithOrder])).toBe(true);
			expect(config.isSuccess([])).toBe(false);
		});

		it('getRemoveOrderConfig should return correct configuration', () => {
			const config = getRemoveOrderConfig('https://test.subgraph.com', 'tx123', 'Order removed');

			expect(config.subgraphUrl).toBe('https://test.subgraph.com');
			expect(config.txHash).toBe('tx123');
			expect(config.successMessage).toBe('Order removed');
			expect(config.fetchData).toBe(getTransactionRemoveOrders);

			expect(config.isSuccess([{ order: { id: 'order1' } } as SgRemoveOrderWithOrder])).toBe(true);
			expect(config.isSuccess([])).toBe(false);
		});
	});
}
