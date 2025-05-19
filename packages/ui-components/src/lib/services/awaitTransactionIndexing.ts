/**
 * @module awaitTransactionIndexing
 * @description Utilities for waiting for transactions to be indexed by a subgraph
 */

import type { WasmEncodedResult } from '@rainlanguage/orderbook';

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
 * @param options.subgraphUrl URL of the subgraph to query
 * @param options.txHash Transaction hash to check for indexing
 * @param options.successMessage Message to display on successful indexing
 * @param options.maxAttempts Maximum number of attempts before timing out (default: 10)
 * @param options.interval Interval between attempts in milliseconds (default: 1000)
 * @param options.network Optional network identifier
 * @param options.fetchEntityFn Function to fetch data from the subgraph
 * @param options.isSuccess Function to determine if the fetched data indicates success
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
	fetchEntityFn: (
		subgraphUrl: string,
		txHash: string
	) => Promise<WasmEncodedResult<T | null | undefined>>;
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
		fetchEntityFn,
		isSuccess
	} = options;

	const checkIndexing = async (attempt: number): Promise<IndexingResult<T>> => {
		try {
			const data = await fetchEntityFn(subgraphUrl, txHash);

			if (data.value && isSuccess(data.value)) {
				let orderHash;
				// Extract orderHash from order data if it exists in the expected format
				if (Array.isArray(data.value) && data.value.length > 0 && data.value[0]?.order?.orderHash) {
					orderHash = data.value[0].order.orderHash;
				}

				return {
					value: {
						txHash,
						successMessage,
						orderHash,
						network,
						data: data.value
					}
				};
			}
		} catch {
			// Continue with the next attempt
		}

		if (attempt >= maxAttempts) {
			return {
				error: TIMEOUT_ERROR
			};
		}

		await new Promise((resolve) => setTimeout(resolve, interval));
		return checkIndexing(attempt + 1);
	};

	return checkIndexing(1);
};
