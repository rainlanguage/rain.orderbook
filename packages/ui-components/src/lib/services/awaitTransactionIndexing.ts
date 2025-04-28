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
					// Extract orderHash from order data if it exists in the expected format
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
