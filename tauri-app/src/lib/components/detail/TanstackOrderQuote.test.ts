import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import TanstackOrderQuote from './TanstackOrderQuote.svelte';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import { mockOrderDetailsExtended } from '$lib/queries/orderDetail';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { writable } = await import('svelte/store');
  const { mockSettingsStore } = await import('$lib/mocks/settings');

  const _activeOrderbook = writable();

  return {
    ...((await importOriginal()) as object),
    settings: mockSettingsStore,
    subgraphUrl: writable('https://example.com'),
    activeOrderbook: {
      ..._activeOrderbook,
      load: vi.fn(() => _activeOrderbook.set(true)),
    },
  };
});

vi.mock('$lib/services/modal', async () => {
  return {
    handleDepositGenericModal: vi.fn(),
    handleDepositModal: vi.fn(),
    handleWithdrawModal: vi.fn(),
  };
});

test('displays order quote data when query is successful', async () => {
  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [
        {
          success: true,
          pair_name: 'ETH/USDT',
          data: { maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' },
          error: undefined,
        },
      ];
    }
  });

  const queryClient = new QueryClient();

  render(TanstackOrderQuote, {
    props: {
      id: '0x123',
      order: mockOrderDetailsExtended.order,
    },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    const orderQuoteComponent = screen.getByTestId('bodyRow');

    expect(orderQuoteComponent).toHaveTextContent('ETH/USDT');
    expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692');
    expect(orderQuoteComponent).toHaveTextContent('6.563567234157974775');
  });
});

test('refreshes the quote when the refresh icon is clicked', async () => {
  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [
        {
          success: true,
          pair_name: 'ETH/USDT',
          data: { maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' },
          error: undefined,
        },
        {
          success: true,
          pair_name: 'BTC/USDT',
          data: { maxOutput: '0x54fa82f5c7001dad', ratio: '0x53e0089714d06709' },
          error: undefined,
        },
      ];
    }
  });

  const queryClient = new QueryClient();

  render(TanstackOrderQuote, {
    props: {
      id: '0x123',
      order: mockOrderDetailsExtended.order,
    },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    const orderQuoteRows = screen.getAllByTestId('bodyRow');

    // Check ETH/USDT row
    expect(orderQuoteRows[0]).toHaveTextContent('1.550122181502135692');

    // Check BTC/USDT row
    expect(orderQuoteRows[1]).toHaveTextContent('6.123350635480882605');
  });

  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [
        {
          success: true,
          pair_name: 'ETH/USD',
          data: { maxOutput: '0x5282713eceeccb5e', ratio: '0x577fe09a8775137c' },
          error: undefined,
        },
        {
          success: true,
          pair_name: 'BTC/USDT',
          data: { maxOutput: '0x5430775053da5e53', ratio: '0x5a01719c871bb83f' },
          error: undefined,
        },
      ];
    }
  });

  const refreshButton = screen.getByTestId('refreshButton');
  fireEvent.click(refreshButton);

  await waitFor(() => {
    const orderQuoteRows = screen.getAllByTestId('bodyRow');

    // Check ETH/USD row
    expect(orderQuoteRows[0]).toHaveTextContent('ETH/USD');
    expect(orderQuoteRows[0]).toHaveTextContent('5.945438972656012126');
    expect(orderQuoteRows[0]).toHaveTextContent('6.305004957644166012');

    // Check BTC/USDT row
    expect(orderQuoteRows[1]).toHaveTextContent('BTC/USDT');
    expect(orderQuoteRows[1]).toHaveTextContent('6.066479884955967059');
    expect(orderQuoteRows[1]).toHaveTextContent('6.485589855485802559');
  });
});

test('displays error message when query fails', async () => {
  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [
        {
          success: false,
          pair_name: 'ETH/USDT',
          data: undefined,
          error: 'Network error',
        },
      ];
    }
  });

  const queryClient = new QueryClient();

  render(TanstackOrderQuote, {
    props: {
      id: '0x123',
      order: mockOrderDetailsExtended.order,
    },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    const errorCell = screen.getByText(
      (content) =>
        content.includes('Error fetching pair quote:') && content.includes('Network error'),
    );
    expect(errorCell).toBeInTheDocument();
  });
});
