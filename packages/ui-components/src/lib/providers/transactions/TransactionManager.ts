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
	type SgTransaction,
	type SgVault,
	type SgOrder
} from '@rainlanguage/orderbook';
import { formatUnits } from 'viem';

/**
 * Function type for adding toast notifications to the UI.
 * @param toast - The toast notification configuration object.
 * @param toast.message - The message text to display in the toast.
 * @param toast.type - The type of toast notification ('success', 'error', or 'info').
 * @param toast.color - The color theme of the toast.
 * @param toast.links - Optional array of links to display in the toast.
 */
export type AddToastFunction = (toast: Omit<ToastProps, 'id'>) => void;

/**
 * Manages blockchain transactions with toast notifications and query invalidation.
 * Handles transaction lifecycle, status updates, and UI feedback.
 * Provides functionality for creating, tracking, and managing blockchain transactions.
 */
export class TransactionManager {
	/** Writable store tracking all active transactions. */
	private transactions: Writable<Transaction[]>;
	/** Query client for cache invalidation after successful transactions. */
	private queryClient: QueryClient;
	/** Function to display toast notifications in the UI. */
	private addToast: AddToastFunction;
	/** Wagmi configuration for blockchain interactions. */
	private wagmiConfig: Config;

	/**
	 * Initializes a new TransactionManager instance.
	 * @param queryClient - Query client for cache invalidation.
	 * @param addToast - Function to display toast notifications.
	 * @param wagmiConfig - Wagmi configuration for blockchain interactions.
	 */
	constructor(queryClient: QueryClient, addToast: AddToastFunction, wagmiConfig: Config) {
		this.queryClient = queryClient;
		this.addToast = addToast;
		this.wagmiConfig = wagmiConfig;
		this.transactions = writable<Transaction[]>([]);
	}

	/**
	 * Creates and initializes a new transaction for removing an order from the orderbook.
	 * @param args - Configuration for the remove order transaction.
	 * @param args.txHash - Hash of the transaction to track.
	 * @param args.chainId - Chain ID where the transaction is being executed.
	 * @param args.networkKey - Network identifier string (e.g., 'mainnet', 'arbitrum').
	 * @param args.queryKey - The hash of the order to be removed (used for query invalidation and UI links).
	 * @param args.subgraphUrl - URL of the subgraph to query for transaction status.
	 * @param args.entity - The `SgOrder` entity associated with this transaction.
	 * @returns A new Transaction instance configured for order removal.
	 * @example
	 * const tx = await manager.createRemoveOrderTransaction({
	 *   txHash: '0x123...',
	 *   chainId: 1,
	 *   subgraphUrl: 'https://api.thegraph.com/subgraphs/name/...',
	 *   networkKey: 'mainnet',
	 *   queryKey: '0x456...', // Order hash
	 *   entity: sgOrderInstance
	 * });
	 */
	public async createRemoveOrderTransaction(
		args: InternalTransactionArgs & { subgraphUrl: string; entity: SgOrder }
	): Promise<Transaction> {
		const name = TransactionName.REMOVAL;
		const errorMessage = 'Order removal failed.';
		const successMessage = 'Order removed successfully.';
		const queryKey = args.queryKey;
		const networkKey = args.networkKey;
		const subgraphUrl = args.subgraphUrl;

		const explorerLink = await getExplorerLink(args.txHash, args.chainId, 'tx');
		const toastLinks: ToastLink[] = [
			{
				link: `/orders/${networkKey}-${args.queryKey}`,
				label: 'View Order'
			},
			{
				link: explorerLink,
				label: 'View on explorer'
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
				subgraphUrl: subgraphUrl,
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
	 * Creates and initializes a new transaction for withdrawing funds from a vault.
	 * @param args - Configuration for the withdrawal transaction.
	 * @param args.txHash - Hash of the transaction to track.
	 * @param args.chainId - Chain ID where the transaction is being executed.
	 * @param args.networkKey - Network identifier string.
	 * @param args.queryKey - The ID of the vault from which funds are withdrawn (used for query invalidation and UI links).
	 * @param args.subgraphUrl - URL of the subgraph to query for transaction status.
	 * @param args.entity - The `SgVault` entity associated with this transaction.
	 * @returns A new Transaction instance configured for withdrawal.
	 * @example
	 * const tx = await manager.createWithdrawTransaction({
	 *   txHash: '0x123...',
	 *   chainId: 1,
	 *   subgraphUrl: 'https://api.thegraph.com/subgraphs/name/...',
	 *   networkKey: 'mainnet',
	 *   queryKey: '0x789...', // Vault ID
	 *   entity: sgVaultInstance
	 * });
	 */
	public async createWithdrawTransaction(
		args: InternalTransactionArgs & { subgraphUrl: string; entity: SgVault }
	): Promise<Transaction> {
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
				label: 'View on explorer'
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
	 * Creates and initializes a new transaction for approving token spend.
	 * @param args - Configuration for the approval transaction.
	 * @param args.txHash - Hash of the transaction to track.
	 * @param args.chainId - Chain ID where the transaction is being executed.
	 * @param args.networkKey - Network identifier string.
	 * @param args.queryKey - The ID of the vault or context for which approval is made (used for query invalidation and UI links).
	 * @param args.entity - The `SgVault` entity (to derive token symbol) associated with this transaction.
	 * @returns A new Transaction instance configured for token approval.
	 * @example
	 * const tx = await manager.createApprovalTransaction({
	 *   txHash: '0xabc...',
	 *   chainId: 1,
	 *   networkKey: 'mainnet',
	 *   queryKey: '0x789...', // Vault ID
	 *   entity: sgVaultInstance
	 * });
	 */
	public async createApprovalTransaction(
		args: InternalTransactionArgs & { entity: SgVault }
	): Promise<Transaction> {
		const tokenSymbol = args.entity.token.symbol;
		const name = `Approving ${tokenSymbol} spend`;
		const errorMessage = 'Approval failed.';
		const successMessage = 'Approval successful.';
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
				label: 'View on explorer'
			}
		];

		return this.createTransaction({
			...args,
			name,
			errorMessage,
			successMessage,
			queryKey,
			toastLinks
		});
	}

	/**
	 * Creates and initializes a new transaction for depositing funds into a vault.
	 * @param args - Configuration for the deposit transaction.
	 * @param args.txHash - Hash of the transaction to track.
	 * @param args.chainId - Chain ID where the transaction is being executed.
	 * @param args.networkKey - Network identifier string.
	 * @param args.queryKey - The ID of the vault into which funds are deposited (used for query invalidation and UI links).
	 * @param args.entity - The `SgVault` entity associated with this transaction.
	 * @param args.amount - The amount of tokens being deposited.
	 * @param args.subgraphUrl - URL of the subgraph to query for transaction status.
	 * @returns A new Transaction instance configured for deposit.
	 * @example
	 * const tx = await manager.createDepositTransaction({
	 *   txHash: '0xdef...',
	 *   chainId: 1,
	 *   subgraphUrl: 'https://api.thegraph.com/subgraphs/name/...',
	 *   networkKey: 'mainnet',
	 *   queryKey: '0x789...', // Vault ID
	 *   entity: sgVaultInstance,
	 *   amount: 1000n
	 * });
	 */
	public async createDepositTransaction(
		args: InternalTransactionArgs & { amount: bigint; entity: SgVault; subgraphUrl: string }
	): Promise<Transaction> {
		const tokenSymbol = args.entity.token.symbol;
		const readableAmount = formatUnits(args.amount, Number(args.entity.token.decimals));
		const name = `Depositing ${readableAmount} ${tokenSymbol}`;
		const errorMessage = 'Deposit failed.';
		const successMessage = 'Deposit successful.';
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
				label: 'View on explorer'
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
	 * Creates, initializes, and executes a new transaction instance.
	 * @param args - Configuration for the transaction.
	 * @param args.name - Name or title of the transaction.
	 * @param args.errorMessage - Message to display on transaction failure.
	 * @param args.successMessage - Message to display on transaction success.
	 * @param args.queryKey - Key used for query invalidation.
	 * @param args.toastLinks - Array of links to display in toast notifications.
	 * @param args.awaitSubgraphConfig - Optional configuration for awaiting subgraph indexing.
	 * @returns A new Transaction instance that has been initialized and started.
	 * @private
	 */
	private async createTransaction(args: TransactionArgs): Promise<Transaction> {
		const createTransactionArgs: TransactionArgs & { config: Config } = {
			...args,
			config: this.wagmiConfig
		};

		const onSuccess = () => {
			this.addToast({
				message: args.successMessage,
				type: 'success',
				color: 'green',
				links: args.toastLinks
			});
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
	 * Retrieves the store containing all active transactions.
	 * @returns A readable store containing all active Transaction instances.
	 * @example
	 * const transactionsStore = manager.getTransactions();
	 * transactionsStore.subscribe(transactions => console.log(transactions));
	 */
	public getTransactions(): Readable<Transaction[]> {
		return this.transactions;
	}

	/**
	 * Removes all transactions from the store, resetting the transaction tracking state.
	 * @example
	 * manager.clearTransactions(); // Clear all tracked transactions
	 */
	public clearTransactions(): void {
		this.transactions.set([]);
	}
}
