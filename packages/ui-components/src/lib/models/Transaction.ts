import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionStoreErrorMessage } from '$lib/types/transaction';
import type { TransactionArgs, TransactionName } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getRemoveOrderConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';
import { getExplorerLink } from '$lib/services/getExplorerLink';
import { match, P } from 'ts-pattern';
import { writable, type Writable } from 'svelte/store';

/**
 * Represents the possible states of a transaction
 * @typedef {Object} TransactionStoreState
 * @property {TransactionName} name - The name of the transaction
 * @property {TransactionStatusMessage} status - The current status of the transaction
 * @property {string} explorerLink - Link to view the transaction on a block explorer
 * @property {Hex} [hash] - Optional transaction hash for successful transactions
 * @property {TransactionStoreErrorMessage} [errorDetails] - Optional error details for failed transactions
 */
export type TransactionStoreState = {
	name: TransactionName;
	status: TransactionStatusMessage;
	explorerLink: string;
	errorDetails?: TransactionStoreErrorMessage;
};

/**
 * Interface defining the structure of a transaction
 * @interface Transaction
 * @property {Writable<TransactionStoreState>} state - A writable store containing the transaction state
 */
export type Transaction = {
	readonly state: Writable<TransactionStoreState>;
};

/**
 * Manages the lifecycle of a transaction including receipt confirmation and subgraph indexing
 * @class TransactionStore
 * @implements {Transaction}
 */
export class TransactionStore implements Transaction {
	private name: TransactionName;
	private config: Config;
	private chainId: number;
	private subgraphUrl: string;
	private txHash: Hex;
	private onSuccess: () => void;
	private onError: () => void;

	public readonly state: Writable<TransactionStoreState>;

	/**
	 * Creates a new TransactionStore instance
	 * @param {TransactionArgs & { config: Config }} args - Configuration arguments for the transaction
	 * @param {() => void} onSuccess - Callback function to execute on successful transaction
	 * @param {() => void} onError - Callback function to execute on failed transaction
	 */
	constructor(
		args: TransactionArgs & { config: Config },
		onSuccess: () => void,
		onError: () => void
	) {
		console.log('[TransactionStore] Initializing with args:', {
			name: args.name,
			chainId: args.chainId,
			txHash: args.txHash
		});

		this.config = args.config;
		this.chainId = args.chainId;
		this.subgraphUrl = args.subgraphUrl;
		this.txHash = args.txHash;
		this.name = args.name;
		this.state = writable<TransactionStoreState>({
			name: this.name,
			status: TransactionStatusMessage.IDLE,
			explorerLink: ''
		});
		this.onSuccess = onSuccess;
		this.onError = onError;
	}

	/**
	 * Updates the transaction state with new values
	 * @param {Partial<TransactionStoreState>} partialState - The new state values to merge with current state
	 * @private
	 */
	private updateState(partialState: Partial<TransactionStoreState>): void {
		console.log('[TransactionStore] Updating state:', partialState);
		this.state.update((currentState) => {
			const newState = {
				...currentState,
				...partialState
			};
			console.log('[TransactionStore] New state:', newState);
			return newState;
		});
	}

	/**
	 * Executes the transaction and begins monitoring its status
	 * @returns {Promise<void>}
	 */
	public async execute(): Promise<void> {
		console.log('[TransactionStore] Executing transaction:', this.txHash);
		const explorerLink = await getExplorerLink(this.txHash, this.chainId, 'tx');
		console.log('[TransactionStore] Generated explorer link:', explorerLink);

		this.updateState({
			explorerLink,
			status: TransactionStatusMessage.PENDING_RECEIPT
		});
		await this.waitForTxReceipt(this.txHash);
	}

	/**
	 * Waits for the transaction receipt to be confirmed on the blockchain
	 * @param {Hex} hash - The transaction hash to monitor
	 * @returns {Promise<void>}
	 * @private
	 */
	private async waitForTxReceipt(hash: Hex): Promise<void> {
		console.log('[TransactionStore] Waiting for transaction receipt:', hash);
		try {
			await waitForTransactionReceipt(this.config, { hash });
			console.log('[TransactionStore] Transaction receipt received');
			await this.indexTransaction(this.txHash);
		} catch (error) {
			console.error('[TransactionStore] Error waiting for receipt:', error);
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionStoreErrorMessage.RECEIPT_FAILED
			});
			return this.onError();
		}
	}

	/**
	 * Monitors the transaction indexing status in the subgraph
	 * @param {Hex} txHash - The transaction hash to monitor
	 * @returns {Promise<void>}
	 * @private
	 */
	private async indexTransaction(txHash: Hex): Promise<void> {
		console.log('[TransactionStore] Starting transaction indexing:', {
			txHash,
			subgraphUrl: this.subgraphUrl,
			name: this.name
		});

		this.updateState({
			status: TransactionStatusMessage.PENDING_SUBGRAPH
		});

		const config = getRemoveOrderConfig(this.subgraphUrl, txHash, 'Order removed successfully');
		console.log('[TransactionStore] Using config for indexing:', {
			subgraphUrl: config.subgraphUrl,
			txHash: config.txHash,
			successMessage: config.successMessage,
			maxAttempts: config.maxAttempts,
			interval: config.interval
		});

		const result = await awaitSubgraphIndexing(config);
		console.log('[TransactionStore] Subgraph indexing result:', {
			hasValue: !!result.value,
			hasError: !!result.error,
			error: result.error,
			value: result.value
				? {
						txHash: result.value.txHash,
						successMessage: result.value.successMessage,
						orderHash: result.value.orderHash,
						network: result.value.network
					}
				: undefined
		});

		if (result.error === TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR) {
			console.error('[TransactionStore] Subgraph timeout error - max attempts reached');
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR
			});
			return this.onError();
		}

		if (result.value) {
			console.log(
				'[TransactionStore] Transaction completed successfully with value:',
				result.value
			);
			this.updateState({
				status: TransactionStatusMessage.SUCCESS
			});
			return this.onSuccess();
		}

		console.error('[TransactionStore] Subgraph indexing failed - no value or error returned');
		this.updateState({
			status: TransactionStatusMessage.ERROR,
			errorDetails: TransactionStoreErrorMessage.SUBGRAPH_FAILED
		});
		return this.onError();
	}
}
