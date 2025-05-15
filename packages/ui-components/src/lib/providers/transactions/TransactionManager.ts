import { writable, type Writable } from 'svelte/store';
import type { QueryClient } from '@tanstack/svelte-query';
import { TransactionStore, type Transaction } from '../../models/Transaction';
import type { InternalTransactionArgs, TransactionArgs } from '$lib/types/transaction';
import type { Config } from '@wagmi/core';
import type { ToastLink, ToastProps } from '$lib/types/toast';
import {
	getTransactionRemoveOrders
} from '@rainlanguage/orderbook';
import { getExplorerLink } from '$lib/services/getExplorerLink';

/**
 * Function type for adding toast notifications to the UI
 * @param toast - The toast notification configuration object
 * @param toast.message - The message text to display in the toast
 * @param toast.type - The type of toast notification (success, error, or info)
 * @param toast.duration - The display duration in milliseconds
 * @param toast.details - Optional additional information or error details
 */
export type AddToastFunction = (toast: ToastProps) => void;

/**
 * Arguments required for removing a transaction
 * @property errorMessage - Message to display on transaction failure
 * @property successMessage - Message to display on transaction success
 * @property queryKey - Key used for query invalidation
 */
export type RemoveTxArgs = Omit<TransactionArgs, 'config'> & {
   errorMessage: string;
   successMessage: string;
   queryKey: string;
};

/**
 * Manages blockchain transactions with toast notifications and query invalidation
 * Handles transaction lifecycle, status updates, and UI feedback
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
     */
    constructor(queryClient: QueryClient, addToast: AddToastFunction, wagmiConfig: Config) {
        console.log('TransactionManager: Initializing');
        this.queryClient = queryClient;
        this.addToast = addToast;
        this.wagmiConfig = wagmiConfig;
        this.transactions = writable<Transaction[]>([]);
        console.log('TransactionManager: Initialized successfully');
    }

    /**
     * Creates and initializes a new transaction for removing an order from the orderbook
     * @param args - Configuration for the remove order transaction
     * @param args.subgraphUrl - URL of the subgraph to query for transaction status
     * @param args.txHash - Hash of the transaction to track
     * @param args.orderHash - Hash of the order to be removed
     * @returns A new Transaction instance configured for order removal
     */
    public async createRemoveOrderTransaction(args: InternalTransactionArgs): Promise<Transaction> {
        const errorMessage = 'Order removal failed.';
        const successMessage = 'Order removed successfully.';
        const queryKey = args.orderHash;
        const explorerLink = await getExplorerLink(args.txHash, args.chainId, 'tx');
        const toastLinks: ToastLink[] = [{
            link: `/orders/${args.orderHash}`,
            label: 'View Order'
        }, 
        {
            link: explorerLink,
            label: 'View transaction'
        }
        ];
        return this.createTransaction({
            ...args,
            errorMessage,
            successMessage,
            queryKey,
            toastLinks,
            fetchEntityFn: () => getTransactionRemoveOrders(args.subgraphUrl, args.txHash)
        });
    }

    /**
     * Creates and executes a new transaction instance
     * @param args - Configuration for the transaction
     * @returns A new Transaction instance that has been initialized and started
     */
    private createTransaction(args: TransactionArgs): Transaction {
        console.log('TransactionManager: Creating remove order transaction', args.orderHash);
        const createTransactionArgs: TransactionArgs = {
            ...args,
            config: this.wagmiConfig,
        };

        const onSuccess = () => {
            console.log("✅ ON SUCCESS, toast and query invalidation");
            this.addToast({ message: args.successMessage, type: 'success', color: 'green', link: args.toastLinks });
            this.queryClient.invalidateQueries({ queryKey: [args.queryKey] });
        };

        const onError = () => {
            this.addToast({ message: args.errorMessage, type: 'error', color: 'red', link: args.toastLinks });
        };

        const transactionInstance = new TransactionStore(
            createTransactionArgs,
            onSuccess,
            onError,
            args.fetchEntityFn
        );

        transactionInstance.execute();
        this.transactions.update(currentTransactions => [...currentTransactions, transactionInstance]); 
        return transactionInstance;
    }

    /**
     * Retrieves the store containing all active transactions
     * @returns A writable store containing all active Transaction instances
     */
    public getTransactions(): Writable<Transaction[]> {
        console.log('TransactionManager: Getting transactions store');
        return this.transactions;
    }

    /**
     * Removes all transactions from the store
     * Resets the transaction tracking state
     */
    public clearTransactions(): void {
        console.log('TransactionManager: Clearing all transactions');
        this.transactions.set([]);
    }
}
