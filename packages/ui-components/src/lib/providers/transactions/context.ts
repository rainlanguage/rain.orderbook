import { getContext, setContext } from 'svelte';
import { TransactionManager } from './TransactionManager';

export const TRANSACTION_MANAGER_CONTEXT_KEY = 'rain:ui-components:transactionManager';

export function setTransactionManagerContext(manager: TransactionManager) {
    setContext(TRANSACTION_MANAGER_CONTEXT_KEY, manager);
}

export function getTransactionManagerContext(): TransactionManager {
    const manager = getContext<TransactionManager | undefined>(TRANSACTION_MANAGER_CONTEXT_KEY);
    if (!manager) {
        throw new Error('TransactionManager not found. Did you forget to setTransactionManagerContext?');
    }
    return manager;
}

