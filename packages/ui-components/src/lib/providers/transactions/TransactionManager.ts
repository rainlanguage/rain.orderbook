import { writable, type Writable } from 'svelte/store';
import type { QueryClient } from '@tanstack/svelte-query';
import { RemoveOrder, type RemoveOrderTransaction } from '../../models/RemoveOrderTransaction';
import type { RemoveOrderTransactionArgs, TransactionErrorMessage } from '$lib/types/transaction';
import type { Config } from '@wagmi/core';

// Assuming addToast has a signature like this, please adjust if different
export type AddToastFunction = (message: string, options?: { type?: 'success' | 'error' | 'info', duration?: number, details?: TransactionErrorMessage | string }) => void;

export class TransactionManager {
    private transactions: Writable<RemoveOrderTransaction[]>;
    private queryClient: QueryClient;
    private addToast: AddToastFunction;
    private wagmiConfig: Config;

    constructor(queryClient: QueryClient, addToast: AddToastFunction, wagmiConfig: Config) {
        this.queryClient = queryClient;
        this.addToast = addToast;
        this.wagmiConfig = wagmiConfig;
        this.transactions = writable<RemoveOrderTransaction[]>([]);
    }

    public createRemoveOrderTransaction(args: Omit<RemoveOrderTransactionArgs, 'config'>): RemoveOrder {
        const transactionArgs: RemoveOrderTransactionArgs = {
            ...args,
            config: this.wagmiConfig,
        };

        const onSuccess = () => {
            this.addToast('Order removed successfully!', { type: 'success' });
            this.queryClient.invalidateQueries({ queryKey: [args.orderHash] });
        };

        const onError = (errorDetails?: TransactionErrorMessage | string) => {
            this.addToast('Order removal failed.', { type: 'error', details: errorDetails });
        };

        const removeOrderInstance = new RemoveOrder(
            transactionArgs,
            onSuccess,
            onError
        );

        this.transactions.update(currentTransactions => [...currentTransactions, removeOrderInstance]); 
        return removeOrderInstance;
    }

    public getTransactions(): Writable<RemoveOrderTransaction[]> {
        return this.transactions;
    }

    public clearTransactions(): void {
        this.transactions.set([]);
    }

}


