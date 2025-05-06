import { render, cleanup, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { writable } from 'svelte/store';
import TransactionsListener from '$lib/components/TransactionsListener.svelte';
import { invalidateTanstackQueries, transactionStore, useToasts } from '@rainlanguage/ui-components';

enum TransactionStatus {
  IDLE = 'Idle',
  SUCCESS = 'Success! Transaction confirmed',
  ERROR = 'Something went wrong',
}

const mockAddToast = vi.fn();
const mockInvalidateTanstackQueries = vi.fn();

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
	useQueryClient: () => ({})
}));

describe('TransactionsListener.svelte', () => {
  const testQueryKey = 'test-query-key';

  beforeEach(() => {
    mockTransactionStore.reset();
    vi.mocked(useToasts).mockReturnValue({
        toasts: writable([]),
        addToast: mockAddToast,
        removeToast: vi.fn()
    })
  });

  afterEach(() => {
    cleanup(); // Clean up DOM after each test
  });

  it('should call addToast and invalidateQueries on transaction success', async () => {
    render(TransactionsListener, { props: { queryKey: testQueryKey } });

    mockTransactionStore.mockSetSubscribeValue({
      status: TransactionStatus.SUCCESS,
      message: 'Transaction was super successful!',
    });
    await waitFor(() => {
    expect(mockAddToast).toHaveBeenCalledTimes(1);
    expect(mockAddToast).toHaveBeenCalledWith({
      message: 'Transaction was super successful!',
      type: 'success',
      color: 'green',
    });

    expect(mockInvalidateTanstackQueries).toHaveBeenCalledTimes(1);
    expect(mockInvalidateTanstackQueries).toHaveBeenCalledWith(mockQueryClient, [testQueryKey]);
    });
  });

  it('should call addToast on transaction error', async () => {
    render(TransactionsListener, { props: { queryKey: testQueryKey } });

    const errorMessage = 'Oh no, an error occurred!';
    mockTransactionStore.mockSetSubscribeValue({
      status: TransactionStatus.ERROR,
      error: errorMessage,
    });

    await waitFor(() => {
    expect(mockAddToast).toHaveBeenCalledTimes(1);
    expect(mockAddToast).toHaveBeenCalledWith({
      message: errorMessage,
      type: 'error',
      color: 'red',
    });
    expect(mockInvalidateTanstackQueries).not.toHaveBeenCalled();
    });
  });

    it('should not call addToast or invalidateQueries for IDLE status', async () => {
        render(TransactionsListener, { props: { queryKey: testQueryKey } });

        expect(mockAddToast).not.toHaveBeenCalled();
        expect(mockInvalidateTanstackQueries).not.toHaveBeenCalled();
    });
});
