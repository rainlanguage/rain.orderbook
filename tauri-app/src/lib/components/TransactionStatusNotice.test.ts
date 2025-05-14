import { render, screen, within } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { writable, type Writable } from 'svelte/store';

import TransactionStatusNoticeComponent from './TransactionStatusNotice.svelte';
import type { TransactionStatusNotice } from '$lib/types/tauriBindings';
import { formatBlockExplorerTransactionUrl } from '$lib/utils/transaction';
import { activeChainHasBlockExplorer } from '$lib/stores/settings';

vi.mock('$lib/stores/settings', () => ({
  activeChainHasBlockExplorer: writable(true),
}));

vi.mock('$lib/utils/transaction', () => ({
  formatBlockExplorerTransactionUrl: vi.fn(),
}));

const createNotice = (
  statusType: TransactionStatusNotice['status']['type'],
  payload?: string,
): TransactionStatusNotice => ({
  id: `test-id-${Date.now()}-${Math.random()}`,
  created_at: new Date().toISOString(),
  label: 'Test Label',
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  status: { type: statusType, payload: payload as any },
});

describe('TransactionStatusNotice.svelte', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (vi.mocked(activeChainHasBlockExplorer) as unknown as Writable<boolean>).set(true);
  });

  it('renders the label correctly', () => {
    const notice = createNotice('Initialized');
    render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });
    expect(screen.getByTestId('notice-label')).toHaveTextContent('Test Label');
  });

  it.each([['Initialized' as const], ['PendingPrepare' as const]])(
    'renders correctly for %s status',
    (statusType) => {
      const notice = createNotice(statusType);
      render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });

      const spinner = screen.getByTestId('status-pending-prepare');
      expect(spinner).toBeInTheDocument();
      expect(screen.getByText('Preparing Transaction')).toBeInTheDocument();
    },
  );

  it('renders correctly for PendingSign status', () => {
    const notice = createNotice('PendingSign');
    render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });

    const section = screen.getByTestId('status-pending-sign');
    expect(section).toBeInTheDocument();
    expect(within(section).getByText('Awaiting Signature')).toBeInTheDocument();
    expect(within(section).getByText(/Please review and sign/)).toBeInTheDocument();
  });

  it('renders correctly for PendingSend status', () => {
    const notice = createNotice('PendingSend');
    render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });

    const spinner = screen.getByTestId('status-pending-send');
    expect(spinner).toBeInTheDocument();
    expect(screen.getByText('Submitting Transaction')).toBeInTheDocument();
    expect(screen.getByText(/Sending and awaiting/)).toBeInTheDocument();
  });

  it('renders correctly for Confirmed status with block explorer enabled', () => {
    const txHash = '0xabc123def456';
    const notice = createNotice('Confirmed', txHash);
    const expectedUrl = `https://explorer.test/tx/${txHash}`;
    (vi.mocked(activeChainHasBlockExplorer) as unknown as Writable<boolean>).set(true);
    (vi.mocked(formatBlockExplorerTransactionUrl) as Mock).mockReturnValue(expectedUrl);

    render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });

    const icon = screen.getByTestId('status-confirmed');
    expect(icon).toBeInTheDocument();
    // The text is a sibling of the icon, not within it
    expect(screen.getByText('Transaction Confirmed')).toBeInTheDocument();
    expect(screen.getByTestId('confirmed-payload')).toHaveTextContent(`Hash: ${txHash}`);

    const link = screen.getByTestId('block-explorer-link');
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', expectedUrl);
    expect(link).toHaveAttribute('target', '_blank');
    expect(vi.mocked(formatBlockExplorerTransactionUrl)).toHaveBeenCalledWith(txHash);
  });

  it('renders correctly for Confirmed status with block explorer disabled', () => {
    const txHash = '0xabc123def456';
    const notice = createNotice('Confirmed', txHash);
    (vi.mocked(activeChainHasBlockExplorer) as unknown as Writable<boolean>).set(false);

    render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });

    const icon = screen.getByTestId('status-confirmed');
    expect(icon).toBeInTheDocument();
    expect(screen.getByText('Transaction Confirmed')).toBeInTheDocument();
    expect(screen.getByTestId('confirmed-payload')).toHaveTextContent(`Hash: ${txHash}`);

    expect(screen.queryByTestId('block-explorer-link')).not.toBeInTheDocument();
    expect(vi.mocked(formatBlockExplorerTransactionUrl)).not.toHaveBeenCalled();
  });

  it('renders correctly for Failed status', () => {
    const errorMsg = 'Execution reverted: Insufficient balance';
    const notice = createNotice('Failed', errorMsg);
    render(TransactionStatusNoticeComponent, { transactionStatusNotice: notice });

    const icon = screen.getByTestId('status-failed');
    expect(icon).toBeInTheDocument();
    expect(screen.getByText('Transaction Failed')).toBeInTheDocument();
    expect(screen.getByTestId('failed-payload')).toHaveTextContent(errorMsg);
  });
});
