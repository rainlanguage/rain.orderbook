import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionStoreErrorMessage } from '$lib/types/transaction';
import type { TransactionArgs, TransactionName } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	type AwaitSubgraphConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';
import { getExplorerLink } from '$lib/services/getExplorerLink';
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
	private txHash: Hex;
	private onSuccess: () => void;
	private onError: () => void;
	private awaitSubgraphConfig: AwaitSubgraphConfig;
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
		this.config = args.config;
		this.chainId = args.chainId;
		this.txHash = args.txHash;
		this.name = args.name;
		this.state = writable<TransactionStoreState>({
			name: this.name,
			status: TransactionStatusMessage.IDLE,
			explorerLink: ''
		});
		this.awaitSubgraphConfig = args.awaitSubgraphConfig;
		this.onSuccess = onSuccess;
		this.onError = onError;
	}

	/**
	 * Updates the transaction state with new values
	 * @param {Partial<TransactionStoreState>} partialState - The new state values to merge with current state
	 * @private
	 */
	private updateState(partialState: Partial<TransactionStoreState>): void {
		this.state.update((currentState) => ({
			...currentState,
			...partialState
		}));
	}

	/**
	 * Executes the transaction and begins monitoring its status
	 * @returns {Promise<void>}
	 */
	public async execute(): Promise<void> {
		const explorerLink = await getExplorerLink(this.txHash, this.chainId, 'tx');

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
		try {
			await waitForTransactionReceipt(this.config, { hash });
			await this.indexTransaction(this.txHash);
		} catch {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionStoreErrorMessage.RECEIPT_FAILED
			});
			return this.onError();
		}
	}

	/**
	 * Monitors the transaction indexing status in the subgraph
	 * @returns {Promise<void>}
	 * @private
	 */
	private async indexTransaction(): Promise<void> {
		this.updateState({
			status: TransactionStatusMessage.PENDING_SUBGRAPH
		});

		const result = await awaitSubgraphIndexing(this.awaitSubgraphConfig);

		if (result.error === TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR
			});
			return this.onError();
		}

		if (result.value) {
			this.updateState({
				status: TransactionStatusMessage.SUCCESS
			});
			return this.onSuccess();
		}

		this.updateState({
			status: TransactionStatusMessage.ERROR,
			errorDetails: TransactionStoreErrorMessage.SUBGRAPH_FAILED
		});
		return this.onError();
	}
}
