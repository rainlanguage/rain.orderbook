import { render, cleanup, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from 'vitest';
import { writable } from 'svelte/store';
import TransactionsListener from '$lib/components/TransactionsListener.svelte';
import {
	invalidateTanstackQueries,
	useToasts,
	TransactionStatus
} from '@rainlanguage/ui-components';
import { QueryClient, useQueryClient } from '@tanstack/svelte-query';

const mockAddToast = vi.fn();

const { mockTransactionStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...(await importOriginal()),
		useToasts: vi.fn(),
		transactionStore: mockTransactionStore,
		invalidateTanstackQueries: vi.fn()
	};
});

vi.mock('@tanstack/svelte-query', () => ({
	useQueryClient: vi.fn()
}));

describe('TransactionsListener.svelte', () => {
	const testQueryKey = 'test-query-key';

	beforeEach(() => {
		vi.clearAllMocks();
		mockTransactionStore.reset?.();
		vi.mocked(useQueryClient).mockReturnValue({
			name: 'test'
		} as unknown as QueryClient);
		vi.mocked(useToasts).mockReturnValue({
			toasts: writable([]),
			addToast: mockAddToast,
			removeToast: vi.fn()
		});
	});

	afterEach(() => {
		cleanup();
	});

	it('should call addToast and invalidateQueries on transaction success', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			message: 'Transaction was super successful!'
		});
		await waitFor(() => {
			expect(mockAddToast).toHaveBeenCalledTimes(1);
			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Transaction was super successful!',
				type: 'success',
				color: 'green'
			});

			expect(invalidateTanstackQueries as Mock).toHaveBeenCalledTimes(1);
			expect(invalidateTanstackQueries).toHaveBeenCalledWith({ name: 'test' }, [testQueryKey]);
		});
	});

	it('should call addToast on transaction error', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		const errorMessage = 'Oh no, an error occurred!';
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.ERROR,
			error: errorMessage
		});

		await waitFor(() => {
			expect(mockAddToast).toHaveBeenCalledTimes(1);
			expect(mockAddToast).toHaveBeenCalledWith({
				message: errorMessage,
				type: 'error',
				color: 'red'
			});
			expect(invalidateTanstackQueries as Mock).not.toHaveBeenCalled();
		});
	});

	it('should not call addToast or invalidateQueries for IDLE status', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		expect(mockAddToast).not.toHaveBeenCalled();
		expect(invalidateTanstackQueries as Mock).not.toHaveBeenCalled();
	});

	it('should handle multiple transaction status changes in rapid succession', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		await mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			message: 'Success 1'
		});

		await mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.ERROR,
			error: 'Error 1'
		});

		await mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			message: 'Success 2'
		});

		await waitFor(() => {
			expect(mockAddToast).toHaveBeenCalledTimes(3);
			expect(mockAddToast).toHaveBeenNthCalledWith(1, {
				message: 'Success 1',
				type: 'success',
				color: 'green'
			});
			expect(mockAddToast).toHaveBeenNthCalledWith(2, {
				message: 'Error 1',
				type: 'error',
				color: 'red'
			});
			expect(mockAddToast).toHaveBeenNthCalledWith(3, {
				message: 'Success 2',
				type: 'success',
				color: 'green'
			});

			expect(invalidateTanstackQueries as Mock).toHaveBeenCalledTimes(2);
			expect(invalidateTanstackQueries).toHaveBeenNthCalledWith(1, { name: 'test' }, [
				testQueryKey
			]);
			expect(invalidateTanstackQueries).toHaveBeenNthCalledWith(2, { name: 'test' }, [
				testQueryKey
			]);
		});
	});

	it('should handle transaction error with undefined or empty message', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.ERROR,
			error: undefined
		});

		await waitFor(() => {
			expect(mockAddToast).toHaveBeenCalledTimes(1);
			expect(mockAddToast).toHaveBeenCalledWith({
				message: undefined,
				type: 'error',
				color: 'red'
			});
		});

		mockAddToast.mockClear();

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.ERROR,
			error: ''
		});

		await waitFor(() => {
			expect(mockAddToast).toHaveBeenCalledTimes(1);
			expect(mockAddToast).toHaveBeenCalledWith({
				message: '',
				type: 'error',
				color: 'red'
			});
		});
		expect(invalidateTanstackQueries as Mock).not.toHaveBeenCalled();
	});

	it('should handle malformed transaction objects gracefully', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockTransactionStore.mockSetSubscribeValue(null as any);
		await waitFor(() => {
			expect(mockAddToast).not.toHaveBeenCalled();
		});

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockTransactionStore.mockSetSubscribeValue(undefined as any);
		await waitFor(() => {
			expect(mockAddToast).not.toHaveBeenCalled();
		});

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockTransactionStore.mockSetSubscribeValue({ message: 'Missing status' } as any);
		await waitFor(() => {
			expect(mockAddToast).not.toHaveBeenCalled();
		});

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockTransactionStore.mockSetSubscribeValue({ status: 'INVALID_STATUS_XYZ' } as any);
		await waitFor(() => {
			expect(mockAddToast).not.toHaveBeenCalled();
		});

		expect(invalidateTanstackQueries as Mock).not.toHaveBeenCalled();
	});
});
