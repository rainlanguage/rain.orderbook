import { render, cleanup, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from 'vitest';
import { writable } from 'svelte/store';
import TransactionsListener from '$lib/components/TransactionsListener.svelte';
import {
	invalidateTanstackQueries,
	useToasts,
	TransactionStatusMessage
} from '@rainlanguage/ui-components';
import { QueryClient, useQueryClient } from '@tanstack/svelte-query';

const mockAddToast = vi.fn();
const mockErrToast = vi.fn();
const testQueryKey = 'test-query-key';

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
	beforeEach(() => {
		vi.clearAllMocks();
		mockTransactionStore.reset?.();
		vi.mocked(useQueryClient).mockReturnValue({
			name: 'test'
		} as unknown as QueryClient);
		vi.mocked(useToasts).mockReturnValue({
			toasts: writable([]),
			addToast: mockAddToast,
			removeToast: vi.fn(),
			errToast: mockErrToast
		});
	});

	afterEach(() => {
		cleanup();
	});

	it('should call addToast and invalidateQueries on transaction success', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.SUCCESS,
			message: 'Transaction was super successful!',
			explorerLink: 'https://etherscan.io/tx/0x123'
		});

		await waitFor(() => {
			expect(mockAddToast).toHaveBeenCalledTimes(1);
			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Transaction was super successful!',
				type: 'success',
				color: 'green',
				links: [
					{
						link: 'https://etherscan.io/tx/0x123',
						label: 'View transaction on explorer'
					}
				]
			});

			expect(invalidateTanstackQueries as Mock).toHaveBeenCalledTimes(1);
			expect(invalidateTanstackQueries).toHaveBeenCalledWith({ name: 'test' }, [testQueryKey]);
		});
	});

	it('should call addToast on transaction error', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		const errorMessage = 'Oh no, an error occurred!';
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.ERROR,
			error: errorMessage,
			explorerLink: 'https://etherscan.io/tx/0x123'
		});

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledTimes(1);
			expect(mockErrToast).toHaveBeenCalledWith(errorMessage);
		});
		expect(invalidateTanstackQueries as Mock).not.toHaveBeenCalled();
	});

	it('should not call addToast or invalidateQueries for IDLE status', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.IDLE,
			message: '',
			explorerLink: ''
		});

		expect(mockAddToast).not.toHaveBeenCalled();
		expect(invalidateTanstackQueries as Mock).not.toHaveBeenCalled();
	});

	it('should handle multiple transaction status changes in rapid succession', async () => {
		render(TransactionsListener, { props: { queryKey: testQueryKey } });

		await mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.SUCCESS,
			message: 'Success 1',
			explorerLink: 'https://etherscan.io/tx/0x123'
		});

		await mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.ERROR,
			error: 'Error 1',
			explorerLink: 'https://etherscan.io/tx/0x123'
		});

		await mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.SUCCESS,
			message: 'Success 2',
			explorerLink: 'https://etherscan.io/tx/0x123'
		});

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledTimes(1);
			expect(mockAddToast).toHaveBeenCalledTimes(2);
			expect(mockAddToast).toHaveBeenNthCalledWith(1, {
				message: 'Success 1',
				type: 'success',
				color: 'green',
				links: [
					{
						link: 'https://etherscan.io/tx/0x123',
						label: 'View transaction on explorer'
					}
				]
			});
			expect(mockAddToast).toHaveBeenNthCalledWith(2, {
				message: 'Success 2',
				type: 'success',
				color: 'green',
				links: [
					{
						link: 'https://etherscan.io/tx/0x123',
						label: 'View transaction on explorer'
					}
				]
			});

			expect(mockErrToast).toHaveBeenNthCalledWith(1, 'Error 1');

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
			status: TransactionStatusMessage.ERROR,
			error: undefined,
			explorerLink: 'https://etherscan.io/tx/0x123'
		});

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledTimes(1);
			expect(mockErrToast).toHaveBeenCalledWith(undefined);
		});

		mockAddToast.mockClear();

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatusMessage.ERROR,
			error: 'Error!'
		});

		await waitFor(() => {
			expect(mockErrToast).toHaveBeenCalledTimes(2);
			expect(mockErrToast).toHaveBeenCalledWith('Error!');
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
