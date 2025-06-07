import { writable, type Readable, type Writable } from 'svelte/store';
import type { QueryClient } from '@tanstack/svelte-query';
import { TransactionStore, type Transaction } from '$lib/models/Transaction';
import type { InternalTransactionArgs, TransactionArgs } from '$lib/types/transaction';
import type { Config } from '@wagmi/core';
import type { ToastLink, ToastProps } from '$lib/types/toast';
import { getExplorerLink } from '$lib/services/getExplorerLink';
import { TransactionName } from '$lib/types/transaction';
import {
	getTransaction,
	getTransactionRemoveOrders,
	type SgRemoveOrderWithOrder,
	type SgTransaction
} from '@rainlanguage/orderbook';

/**
 * Function type for adding toast notifications to the UI
 * @param toast - The toast notification configuration object
 * @param toast.message - The message text to display in the toast
 * @param toast.type - The type of toast notification (success, error, or info)
 * @param toast.color - The color theme of the toast
 * @param toast.links - Optional array of links to display in the toast
 */
export type AddToastFunction = (toast: Omit<ToastProps, 'id'>) => void;

/**
 * Manages blockchain transactions with toast notifications and query invalidation.
 * Handles transaction lifecycle, status updates, and UI feedback.
 * Provides functionality for creating, tracking, and managing blockchain transactions.
 *
 * @class TransactionManager
 * @description A class that manages the lifecycle of blockchain transactions, including
 * tracking their status, displaying notifications, and invalidating relevant queries.
 */
export class TransactionManager {
	/** Writable store tracking all active transactions */
	private transactions: Writable<Transaction[]>;
	/** Query client for cache invalidation after successful transactions */
	private queryClient: QueryClient;
	/** Function to display toast notifications in the UI */
	private addToast: AddToastFunction;
	/** Wagmi configuration for blockchain interactions */
	private wagmiConfig: Config;

	/**
	 * Initializes a new TransactionManager instance
	 * @param queryClient - Query client for cache invalidation
	 * @param addToast - Function to display toast notifications
	 * @param wagmiConfig - Wagmi configuration for blockchain interactions
	 * @throws {Error} If required dependencies are not provided
	 */
	constructor(queryClient: QueryClient, addToast: AddToastFunction, wagmiConfig: Config) {
		this.queryClient = queryClient;
		this.addToast = addToast;
		this.wagmiConfig = wagmiConfig;
		this.transactions = writable<Transaction[]>([]);
	}

	/**
	 * Creates and initializes a new transaction for removing an order from the orderbook
	 * @param args - Configuration for the remove order transaction
	 * @param args.subgraphUrl - URL of the subgraph to query for transaction status
	 * @param args.txHash - Hash of the transaction to track
	 * @param args.orderHash - Hash of the order to be removed
	 * @param args.chainId - Chain ID where the transaction is being executed
	 * @param args.queryKey - Key for query invalidation and UI links, often the order hash.
	 * @param args.networkKey - Network identifier for UI links.
	 * @returns A new Transaction instance configured for order removal
	 * @throws {Error} If required transaction parameters are missing
	 * @example
	 * const tx = await manager.createRemoveOrderTransaction({
	 *   subgraphUrl: 'https://api.thegraph.com/subgraphs/name/...',
	 *   txHash: '0x123...',
	 *   orderHash: '0x456...',
	 *   chainId: 1
	 * });
	 */
	public async createRemoveOrderTransaction(args: InternalTransactionArgs): Promise<Transaction> {
		const name = TransactionName.REMOVAL;
		const errorMessage = 'Order removal failed.';
		const successMessage = 'Order removed successfully.';
		const queryKey = args.queryKey;
		const networkKey = args.networkKey;

		const explorerLink = await getExplorerLink(args.txHash, args.chainId, 'tx');
		const toastLinks: ToastLink[] = [
			{
				link: `/orders/${networkKey}-${args.queryKey}`,
				label: 'View Order'
			},
			{
				link: explorerLink,
				label: 'View transaction'
			}
		];
		return this.createTransaction({
			...args,
			name,
			errorMessage,
			successMessage,
			queryKey,
			toastLinks,
			awaitSubgraphConfig: {
				subgraphUrl: args.subgraphUrl,
				txHash: args.txHash,
				successMessage,
				fetchEntityFn: getTransactionRemoveOrders,
				isSuccess: (data: SgRemoveOrderWithOrder[] | SgTransaction) => {
					return Array.isArray(data) ? data.length > 0 : false;
				}
			}
		});
	}

	/**
	 * Creates and initializes a new transaction for withdrawing funds from a vault
	 * @param args - Configuration for the withdrawal transaction
	 * @param args.subgraphUrl - URL of the subgraph to query for transaction status
	 * @param args.txHash - Hash of the transaction to track
	 * @param args.orderHash - Identifier related to the transaction (e.g. vault ID or context-specific ID if applicable).
	 * @param args.chainId - Chain ID where the transaction is being executed
	 * @param args.queryKey - Identifier for the vault, used for cache invalidation and UI links.
	 * @param args.networkKey - Network identifier for UI links.
	 * @returns A new Transaction instance configured for withdrawal
	 * @throws {Error} If required transaction parameters are missing
	 * @example
	 * const tx = await manager.createWithdrawTransaction({
	 *   subgraphUrl: 'https://api.thegraph.com/subgraphs/name/...',
	 *   txHash: '0x123...',
	 *   orderHash: '0x456...',
	 *   chainId: 1
	 * });
	 */
	public async createWithdrawTransaction(args: InternalTransactionArgs): Promise<Transaction> {
		const name = TransactionName.WITHDRAWAL;
		const errorMessage = 'Withdrawal failed.';
		const successMessage = 'Withdrawal successful.';
		const queryKey = args.queryKey;
		const networkKey = args.networkKey;

		const explorerLink = await getExplorerLink(args.txHash, args.chainId, 'tx');
		const toastLinks: ToastLink[] = [
			{
				link: `/vaults/${networkKey}-${args.queryKey}`,
				label: 'View vault'
			},
			{
				link: explorerLink,
				label: 'View transaction'
			}
		];
		return this.createTransaction({
			...args,
			name,
			errorMessage,
			successMessage,
			queryKey,
			toastLinks,
			awaitSubgraphConfig: {
				subgraphUrl: args.subgraphUrl,
				txHash: args.txHash,
				successMessage,
				fetchEntityFn: getTransaction,
				isSuccess: (data) => !!data
			}
		});
	}

	/**
	 * Creates and executes a new transaction instance
	 * @param args - Configuration for the transaction
	 * @param args.pendingMessage - Message to display on pending transaction
	 * @param args.errorMessage - Message to display on transaction failure
	 * @param args.successMessage - Message to display on transaction success
	 * @param args.queryKey - Key used for query invalidation
	 * @param args.toastLinks - Array of links to display in toast notifications
	 * @returns A new Transaction instance that has been initialized and started
	 * @throws {Error} If transaction creation fails
	 * @private
	 */
	private async createTransaction(args: TransactionArgs): Promise<Transaction> {
		const createTransactionArgs: TransactionArgs & { config: Config } = {
			...args,
			config: this.wagmiConfig
		};

		const onSuccess = () => {
			this.queryClient.invalidateQueries({ queryKey: [args.queryKey] });
		};

		const onError = () => {
			this.addToast({
				message: args.errorMessage,
				type: 'error',
				color: 'red',
				links: args.toastLinks
			});
		};

		const transactionInstance = new TransactionStore(createTransactionArgs, onSuccess, onError);

		this.transactions.update((currentTransactions) => [
			...currentTransactions,
			transactionInstance
		]);
		await transactionInstance.execute();
		return transactionInstance;
	}

	/**
	 * Retrieves the store containing all active transactions
	 * @returns A writable store containing all active Transaction instances
	 * @throws {Error} If transaction store is not initialized
	 * @example
	 * const transactions = manager.getTransactions();
	 * $: console.log($transactions); // Log all active transactions
	 */
	public getTransactions(): Readable<Transaction[]> {
		return this.transactions;
	}

	/**
	 * Removes all transactions from the store
	 * Resets the transaction tracking state
	 * @throws {Error} If transaction store is not initialized
	 * @example
	 * manager.clearTransactions(); // Clear all tracked transactions
	 */
	public clearTransactions(): void {
		this.transactions.set([]);
	}
}
