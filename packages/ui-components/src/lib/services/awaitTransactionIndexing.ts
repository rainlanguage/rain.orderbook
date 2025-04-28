/**
 * @module awaitTransactionIndexing
 * @description Utilities for waiting for transactions to be indexed by a subgraph
 */

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

/**
 * Error message when subgraph indexing times out
 */
export const TIMEOUT_ERROR =
	'The subgraph took too long to respond. Your transaction may still be processing.';

/**
 * Result of a subgraph indexing operation
 * @template T The type of data returned by the subgraph
 */
export type IndexingResult<T> = {
	/**
	 * The successful result of the indexing operation
	 */
	value?: {
		/**
		 * The transaction hash
		 */
		txHash: string;
		/**
		 * Message to display on successful indexing
		 */
		successMessage: string;
		/**
		 * Optional order hash if available
		 */
		orderHash?: string;
		/**
		 * Optional network key
		 */
		network?: string;
		/**
		 * Optional data returned from the subgraph
		 */
		data?: T;
	};
	/**
	 * Error message if indexing failed
	 */
	error?: string;
};

/**
 * Generic function to handle waiting for subgraph indexing
 * Returns a promise that resolves with an object containing either value or error
 * 
 * @template T The type of data returned by the subgraph
 * @param options Configuration options for the indexing operation
 * @returns Promise resolving to an IndexingResult
 */
export const awaitSubgraphIndexing = async <T>(options: {
	/**
	 * URL of the subgraph to query
	 */
	subgraphUrl: string;
	/**
	 * Transaction hash to check for indexing
	 */
	txHash: string;
	/**
	 * Message to display on successful indexing
	 */
	successMessage: string;
	/**
	 * Maximum number of attempts before timing out
	 */
	maxAttempts?: number;
	/**
	 * Interval between attempts in milliseconds
	 */
	interval?: number;
	/**
	 * Optional network identifier
	 */
	network?: string;
	/**
	 * Function to fetch data from the subgraph
	 * @param subgraphUrl URL of the subgraph
	 * @param txHash Transaction hash to query
	 */
	fetchData: (subgraphUrl: string, txHash: string) => Promise<T | null | undefined>;
	/**
	 * Function to determine if the fetched data indicates success
	 * @param data The data returned from the subgraph
	 */
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

/**
 * Configuration for transaction indexing
 * @template T The type of data returned by the subgraph
 */
export interface TransactionConfig<T> {
	/**
	 * URL of the subgraph to query
	 */
	subgraphUrl: string;
	/**
	 * Transaction hash to check for indexing
	 */
	txHash: string;
	/**
	 * Message to display on successful indexing
	 */
	successMessage: string;
	/**
	 * Optional network key
	 */
	network?: string;
	/**
	 * Maximum number of attempts before timing out
	 */
	maxAttempts?: number;
	/**
	 * Interval between attempts in milliseconds
	 */
	interval?: number;
	/**
	 * Function to fetch data from the subgraph
	 */
	fetchData: (subgraphUrl: string, txHash: string) => Promise<T>;
	/**
	 * Function to determine if the fetched data indicates success
	 */
	isSuccess: (data: T) => boolean;
}

/**
 * Creates a configuration for checking general transaction indexing
 * 
 * @param subgraphUrl URL of the subgraph to query
 * @param txHash Transaction hash to check for indexing
 * @param successMessage Message to display on successful indexing
 * @param network Optional network key
 * @param maxAttempts Maximum number of attempts before timing out
 * @param interval Interval between attempts in milliseconds
 * @returns Configuration for transaction indexing
 */
export const getTransactionConfig = (
	subgraphUrl: string,
	txHash: string,
	successMessage: string,
	network?: string,
	maxAttempts?: number,
	interval?: number
): TransactionConfig<SgTransaction> => ({
	subgraphUrl,
	txHash,
	successMessage,
	network,
	maxAttempts,
	interval,
	fetchData: getTransaction,
	isSuccess: (tx: SgTransaction) => !!tx
});

/**
 * Creates a configuration for checking new order indexing
 * 
 * @param subgraphUrl URL of the subgraph to query
 * @param txHash Transaction hash to check for indexing
 * @param successMessage Message to display on successful indexing
 * @param network Optional network key
 * @param maxAttempts Maximum number of attempts before timing out
 * @param interval Interval between attempts in milliseconds
 * @returns Configuration for new order indexing
 */
export const getNewOrderConfig = (
	subgraphUrl: string,
	txHash: string,
	successMessage: string,
	network?: string,
	maxAttempts?: number,
	interval?: number
): TransactionConfig<SgAddOrderWithOrder[]> => ({
	subgraphUrl,
	txHash,
	successMessage,
	network,
	maxAttempts,
	interval,
	fetchData: getTransactionAddOrders,
	isSuccess: (addOrders: SgAddOrderWithOrder[]) => addOrders?.length > 0
});

/**
 * Creates a configuration for checking order removal indexing
 * 
 * @param subgraphUrl URL of the subgraph to query
 * @param txHash Transaction hash to check for indexing
 * @param successMessage Message to display on successful indexing
 * @param network Optional network key
 * @param maxAttempts Maximum number of attempts before timing out
 * @param interval Interval between attempts in milliseconds
 * @returns Configuration for order removal indexing
 */
export const getRemoveOrderConfig = (
	subgraphUrl: string,
	txHash: string,
	successMessage: string,
	network?: string,
	maxAttempts?: number,
	interval?: number
): TransactionConfig<SgRemoveOrderWithOrder[]> => ({
	subgraphUrl,
	txHash,
	successMessage,
	network,
	maxAttempts,
	interval,
	fetchData: getTransactionRemoveOrders,
	isSuccess: (removeOrders: SgRemoveOrderWithOrder[]) => removeOrders?.length > 0
});
