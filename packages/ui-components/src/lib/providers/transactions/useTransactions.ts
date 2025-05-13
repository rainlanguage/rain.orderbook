import type { BaseTransaction } from '$lib/types/transaction';
import { get } from 'svelte/store';
import { getTransactionsContext } from './context';

/**
 * Hook for managing transactions in the application.
 * Provides functionality to add, remove, and access transactions.
 *
 * @returns An object containing the transactions store and methods to manipulate transactions
 */
export function useTransactions() {
	const transactions = getTransactionsContext();

    // Add a transaction to the store
    const addTransaction = (transaction: BaseTransaction) => {
        transactions.update((transactions) => [...transactions, transaction]);
    }

    // Lookup a transaction by its hash
    const getTransaction = (hash: string) => {
        return get(transactions).find((transaction) => transaction.state.hash === hash);
    }

	return {
		addTransaction,
		getTransaction,
	};
}
