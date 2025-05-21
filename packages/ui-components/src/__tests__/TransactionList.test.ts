import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import TransactionList from '../lib/components/transactions/TransactionList.svelte';
import { TransactionManager } from '../lib/providers/transactions/TransactionManager';
import { writable } from 'svelte/store';
import type { Transaction, TransactionStoreState } from '../lib/models/Transaction';
import { TransactionStatusMessage, TransactionName } from '../lib/types/transaction';
import { useTransactions } from '../lib/providers/transactions/useTransactions';

vi.mock('$lib/components/transaction/TransactionDetail.svelte', async () => {
	const mockTransactionDetail = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockTransactionDetail };
});

vi.mock('../lib/providers/transactions/useTransactions', () => ({
	useTransactions: vi.fn()
}));

describe('TransactionList', () => {
	let mockTransactionManager: TransactionManager;
	let mockTransactionsStore: ReturnType<typeof writable<Transaction[]>>;

	beforeEach(() => {
		vi.clearAllMocks();

		mockTransactionsStore = writable<Transaction[]>([]);

		mockTransactionManager = {
			transactions: vi.fn().mockReturnValue(mockTransactionsStore)
		} as unknown as TransactionManager;

		vi.mocked(useTransactions).mockReturnValue({
			manager: mockTransactionManager,
			transactions: mockTransactionsStore
		});
	});

	it('should not render anything when there are no transactions', () => {
		const { container } = render(TransactionList);
		expect(container.innerHTML).toBe('');
	});

	it('should render a list of transactions when transactions exist', () => {
		const mockTransactions = [
			{
				state: writable<TransactionStoreState>({
					status: TransactionStatusMessage.IDLE,
					name: TransactionName.REMOVAL,
					links: [
						{
							label: 'View on Explorer',
							link: 'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
						}
					]
				})
			},
			{
				state: writable<TransactionStoreState>({
					status: TransactionStatusMessage.SUCCESS,
					name: TransactionName.REMOVAL,
					links: [
						{
							label: 'View on Explorer',
							link: 'https://etherscan.io/tx/0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
						}
					]
				})
			}
		] as unknown as Transaction[];

		mockTransactionsStore.set(mockTransactions);

		render(TransactionList);

		const list = screen.getByRole('list');
		expect(list).toBeInTheDocument();
		expect(list.children).toHaveLength(2);
	});

	it('should update when transactions change', async () => {
		const { container } = render(TransactionList);

		expect(container.innerHTML).toBe('');

		mockTransactionsStore.set([
			{
				state: writable<TransactionStoreState>({
					status: TransactionStatusMessage.IDLE,
					name: TransactionName.REMOVAL,
					links: [
						{
							label: 'View on Explorer',
							link: 'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
						}
					]
				})
			}
		] as unknown as Transaction[]);

		await waitFor(() => {
			expect(screen.getByRole('list')).toBeInTheDocument();
			expect(screen.getByRole('list').children).toHaveLength(1);
		});

		mockTransactionsStore.set([]);

		await waitFor(() => {
			expect(container.innerHTML).toBe('');
		});
	});

	it('should get transactions from the manager', () => {
		render(TransactionList);
		expect(useTransactions).toHaveBeenCalled();
	});
});
