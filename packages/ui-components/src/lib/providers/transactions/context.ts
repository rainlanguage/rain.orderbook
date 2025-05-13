import type { BaseTransaction } from '$lib/types/transaction';
import { getContext, setContext } from 'svelte';
import { type Writable } from 'svelte/store';

export const TRANSACTIONS_KEY = 'rain:ui-components:transactions';

/**
 * Retrieves the transactions store from Svelte's context
 */
export function getTransactionsContext(): Writable<BaseTransaction[]> {
	const transactions = getContext<Writable<BaseTransaction[]>>(TRANSACTIONS_KEY);
	if (!transactions) {
		throw new Error(
			'No toasts context found. Did you forget to wrap your component with ToastProvider?'
		);
	}
	return transactions;
}

/**
 * Sets the transactions store in Svelte's context
 */
export function setTransactionsContext(transactions: Writable<BaseTransaction[]>) {
	setContext(TRANSACTIONS_KEY, transactions);
}
