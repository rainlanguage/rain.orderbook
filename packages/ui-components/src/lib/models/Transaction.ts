import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionStoreErrorMessage } from '$lib/types/transaction';
import type { TransactionArgs, TransactionName } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getRemoveOrderConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';
import { writable, type Writable } from 'svelte/store';

/**
 * Represents the state of a transaction.
 * @typedef {Object} TransactionStoreState
 * @property {TransactionName} name - The user-friendly name of the transaction (e.g., "Order Removal").
 * @property {TransactionStatusMessage} status - The current status of the transaction (e.g., PENDING_RECEIPT, SUCCESS, ERROR).
 * @property {TransactionStoreErrorMessage} [errorDetails] - Optional error message if the transaction failed.
 * @property {Array<{link: string, label: string}>} links - An array of relevant links for the transaction (e.g., explorer link, link to the affected entity).
 */
export type TransactionStoreState = {
	name: TransactionName;
	status: TransactionStatusMessage;
	errorDetails?: TransactionStoreErrorMessage;
	links: {
		link: string;
		label: string;
	}[];
};

/**
 * Interface defining the structure of a transaction object.
 * @interface Transaction
 * @property {Writable<TransactionStoreState>} state - A writable store holding the current state of the transaction.
 */
export type Transaction = {
	readonly state: Writable<TransactionStoreState>;
};

/**
 * Manages the lifecycle of an individual transaction, including waiting for
 * blockchain confirmation (receipt) and subgraph indexing.
 * It exposes its state as a Svelte writable store.
 * @class TransactionStore
 * @implements {Transaction}
 */
export class TransactionStore implements Transaction {
	private name: TransactionName;
	private config: Config;
	private chainId: number;
	private subgraphUrl: string;
	private txHash: Hex;
	private links: {
		link: string;
		label: string;
	}[];
	private onSuccess: () => void;
	private onError: () => void;

	public readonly state: Writable<TransactionStoreState>;

	/**
	 * Creates a new TransactionStore instance.
	 * @param {TransactionArgs & { config: Config }} args - Configuration arguments for the transaction, including the wagmi `Config`.
	 * @param {() => void} onSuccess - Callback invoked when the transaction successfully completes (including indexing).
	 * @param {() => void} onError - Callback invoked if the transaction fails at any stage.
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
		this.links = args.toastLinks;
		this.chainId = args.chainId;
		this.state = writable<TransactionStoreState>({
			name: this.name,
			status: TransactionStatusMessage.IDLE,
			links: this.links
		});
		this.onSuccess = onSuccess;
		this.onError = onError;
	}

	/**
	 * Updates the internal Svelte store with new state values.
	 * @param {Partial<TransactionStoreState>} partialState - An object containing the state properties to update.
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
		this.updateState({
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
	 * @param {Hex} txHash - The transaction hash to monitor
	 * @returns {Promise<void>}
	 * @private
	 */
	private async indexTransaction(txHash: Hex): Promise<void> {
		this.updateState({
			status: TransactionStatusMessage.PENDING_SUBGRAPH
		});

		const config = getRemoveOrderConfig(this.subgraphUrl, txHash, 'Order removed successfully');
		const result = await awaitSubgraphIndexing(config);

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
