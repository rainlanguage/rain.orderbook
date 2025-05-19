import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionErrorMessage } from '$lib/types/transaction';
import type { TransactionArgs, TransactionName } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getRemoveOrderConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';
import { getExplorerLink } from '$lib/services/getExplorerLink';
import { match, P } from 'ts-pattern';
import { writable, type Writable, get } from 'svelte/store';

/**
 * Represents the possible states of a transaction
 * @typedef {Object} TransactionState
 * @property {TransactionName} name - The name of the transaction
 * @property {TransactionStatusMessage} status - The current status of the transaction
 * @property {string} explorerLink - Link to view the transaction on a block explorer
 * @property {Hex} [hash] - Optional transaction hash for successful transactions
 * @property {TransactionErrorMessage} [errorDetails] - Optional error details for failed transactions
 */
export type TransactionState = {
	name: TransactionName;
	status: TransactionStatusMessage;
	explorerLink: string;
	errorDetails?: TransactionErrorMessage;
};

/**
 * Interface defining the structure of a transaction
 * @interface Transaction
 * @property {Writable<TransactionState>} state - A writable store containing the transaction state
 */
export type Transaction = {
	readonly state: Writable<TransactionState>;
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

	public readonly state: Writable<TransactionState>;

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
		this.subgraphUrl = args.subgraphUrl;
		this.txHash = args.txHash;
		this.name = args.name;
		this.state = writable<TransactionState>({
			status: TransactionStatusMessage.IDLE,
			explorerLink: '',
			name: this.name
		});
		this.onSuccess = onSuccess;
		this.onError = onError;
	}

	/**
	 * Updates the transaction state with new values
	 * @param {Partial<TransactionState>} partialState - The new state values to merge with current state
	 * @private
	 */
	private updateState(partialState: Partial<TransactionState>): void {
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
			this.indexTransaction(this.txHash);
		} catch {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.RECEIPT_FAILED
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
		this.updateState({
			status: TransactionStatusMessage.PENDING_SUBGRAPH
		});


			const result = await awaitSubgraphIndexing(
				getRemoveOrderConfig(this.subgraphUrl, txHash, 'Order removed successfully')
			);

			await match(result)
				.with({ error: TransactionErrorMessage.SUBGRAPH_TIMEOUT_ERROR }, () => {
					this.updateState({
						status: TransactionStatusMessage.ERROR,
						errorDetails: TransactionErrorMessage.SUBGRAPH_TIMEOUT_ERROR
					});
					return this.onError();
				})
				.with({ value: P.not(P.nullish) }, () => {
					this.updateState({
						status: TransactionStatusMessage.SUCCESS
					});
					return this.onSuccess();
				})
				.otherwise(() => {
					this.updateState({
						status: TransactionStatusMessage.ERROR,
						errorDetails: TransactionErrorMessage.SUBGRAPH_FAILED
					});
					return this.onError();
				});

	}
}
