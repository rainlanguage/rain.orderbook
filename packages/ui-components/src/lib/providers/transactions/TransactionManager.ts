import { writable, type Writable } from 'svelte/store';
import type { QueryClient } from '@tanstack/svelte-query';
import { RemoveOrder, type RemoveOrderTransaction } from '../../models/RemoveOrderTransaction';
import type { TransactionArgs } from '$lib/types/transaction';
import type { Config } from '@wagmi/core';
import type { ToastProps } from '$lib/types/toast';

/**
 * Function type for adding toast notifications
 * @param toast - The toast notification object to be displayed
 * @param toast.message - The message to display in the toast
 * @param toast.type - The type of toast (success, error, or info)
 * @param toast.duration - How long the toast should be displayed (in milliseconds)
 * @param toast.details - Additional details or error information
 */
export type AddToastFunction = (toast: ToastProps) => void;

/**
 * Manages blockchain transactions with toast notifications and query invalidation
 */
export class TransactionManager {
    /** Store containing all active transactions */
    private transactions: Writable<RemoveOrderTransaction[]>;
    /** Query client for invalidating queries after successful transactions */
    private queryClient: QueryClient;
    /** Function to display toast notifications */
    private addToast: AddToastFunction;
    /** Wagmi configuration for blockchain interactions */
    private wagmiConfig: Config;

    /**
     * Creates a new TransactionManager instance
     * @param queryClient - The query client for cache invalidation
     * @param addToast - Function to display toast notifications
     * @param wagmiConfig - Wagmi configuration for blockchain interactions
     */
    constructor(queryClient: QueryClient, addToast: AddToastFunction, wagmiConfig: Config) {
        console.log('TransactionManager: Initializing');
        this.queryClient = queryClient;
        this.addToast = addToast;
        this.wagmiConfig = wagmiConfig;
        this.transactions = writable<RemoveOrderTransaction[]>([]);
        console.log('TransactionManager: Initialized successfully');
    }

    /**
     * Creates a new transaction for removing an order
     * @param args - Arguments needed for the remove order transaction
     * @returns A new RemoveOrder transaction instance
     */
    public createRemoveOrderTransaction(args: Omit<TransactionArgs, 'config'>): RemoveOrder {
        console.log('TransactionManager: Creating remove order transaction', args.orderHash);
        const transactionArgs: TransactionArgs = {
            ...args,
            config: this.wagmiConfig,
        };

        const onSuccess = () => {
            console.log('TransactionManager: Order removal successful', args.orderHash);
            this.addToast({ message: 'Order removed successfully!', type: 'success', color: 'green' });
            this.queryClient.invalidateQueries({ queryKey: [args.orderHash] });
        };

        const onError = () => {
            console.log('TransactionManager: Order removal failed', args.orderHash);
            this.addToast({ message: 'Order removal failed.', type: 'error', color: 'red' });
        };

        const removeOrderInstance = new RemoveOrder(
            transactionArgs,
            onSuccess,
            onError
        );
        removeOrderInstance.execute();
        console.log('TransactionManager: Adding transaction to store');
        this.transactions.update(currentTransactions => [...currentTransactions, removeOrderInstance]); 
    }

    /**
     * Gets the store containing all active transactions
     * @returns A writable store of RemoveOrderTransaction instances
     */
    public getTransactions(): Writable<RemoveOrderTransaction[]> {
        console.log('TransactionManager: Getting transactions store');
        return this.transactions;
    }

    /**
     * Clears all transactions from the store
     */
    public clearTransactions(): void {
        console.log('TransactionManager: Clearing all transactions');
        this.transactions.set([]);
    }

}
