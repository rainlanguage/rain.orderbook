import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionErrorMessage } from '$lib/types/transaction';
import type { TransactionArgs } from '$lib/types/transaction';
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
 * @property {TransactionStatusMessage} status - The current status of the transaction
 * @property {string} message - A descriptive message about the current state
 * @property {string} explorerLink - Link to view the transaction on a block explorer
 * @property {Hex} [hash] - Optional transaction hash for successful transactions
 * @property {TransactionErrorMessage} [errorDetails] - Optional error details for failed transactions
 */
export type TransactionState =
	| { status: TransactionStatusMessage.IDLE; message: string; explorerLink: string }
	| { status: TransactionStatusMessage.PENDING_REMOVE_ORDER; message: string; explorerLink: string }
	| { status: TransactionStatusMessage.PENDING_SUBGRAPH; message: string; explorerLink: string }
	| { status: TransactionStatusMessage.SUCCESS; message: string; explorerLink: string; hash?: Hex }
	| {
			status: TransactionStatusMessage.ERROR;
			message: string;
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
 * Manages the lifecycle of a blockchain transaction
 * @class TransactionStore
 * @implements {Transaction}
 */
export class TransactionStore implements Transaction {
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
		this.state = writable<TransactionState>({
			status: TransactionStatusMessage.IDLE,
			message: '',
			explorerLink: ''
		});
		this.onSuccess = onSuccess;
		this.onError = onError;
	}

	/**
	 * Gets the current message from the transaction state
	 * @returns {string} The current transaction message
	 */
	public get message(): string {
		return get(this.state).message;
	}

	/**
	 * Updates the transaction state with new values
	 * @param {Partial<TransactionState>} partialState - The new state values to merge
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
			status: TransactionStatusMessage.IDLE,
			message: 'Starting order removal.',
			explorerLink
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
			this.updateState({
				status: TransactionStatusMessage.PENDING_REMOVE_ORDER,
				message: `Waiting for transaction receipt...`
			});

			await waitForTransactionReceipt(this.config, { hash });

			this.updateState({
				message: 'Transaction receipt received.'
			});

			this.indexTransaction(this.txHash);
		} catch {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				message: 'Failed to get transaction receipt.'
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
			status: TransactionStatusMessage.PENDING_SUBGRAPH,
			message: 'Waiting for order removal to be indexed...'
		});

		try {
			const result = await awaitSubgraphIndexing(
				getRemoveOrderConfig(this.subgraphUrl, txHash, 'Order removed successfully')
			);

			await match(result)
				.with({ error: TransactionErrorMessage.TIMEOUT }, () => {
					this.updateState({
						status: TransactionStatusMessage.ERROR,
						message: 'Subgraph indexing timed out.'
					});
					return this.onError();
				})
				.with({ value: P.not(P.nullish) }, ({ value }) => {
					this.updateState({
						status: TransactionStatusMessage.SUCCESS,
						hash: value.txHash as Hex,
						message: 'Order removal indexed successfully.'
					});
					return this.onSuccess();
				})
				.otherwise(() => {
					this.updateState({
						status: TransactionStatusMessage.ERROR,
						message: 'Unknown error during indexing.'
					});
					return this.onError();
				});
		} catch {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.SUBGRAPH_FAILED,
				message: 'Failed to index order removal.'
			});
			return this.onError();
		}
	}
}
