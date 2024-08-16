import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import TanstackOrderQuote from './TanstackOrderQuote.svelte';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';

vi.mock('$lib/services/order', () => ({
  batchOrderQuotes: vi.fn(),
}));

test('displays order quote data when query is successful', async () => {
  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [{ maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' }];
    }
  });

  const queryClient = new QueryClient();

  render(TanstackOrderQuote, {
    props: { orderHash: '0x123' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    const orderQuoteComponent = screen.getByTestId('orderQuoteComponent');

    expect(orderQuoteComponent).toHaveTextContent('Maximum output');
    expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692');
    expect(orderQuoteComponent).toHaveTextContent('Price');
    expect(orderQuoteComponent).toHaveTextContent('6.563567234157974775');
  });
});

test('displays empty message when no data is returned', async () => {
  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [];
    }
  });

  const queryClient = new QueryClient();

  render(TanstackOrderQuote, {
    props: { orderHash: '0x123' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.getByText('Max output and price not found')).toBeInTheDocument();
  });
});

test('refreshes the quote when the refresh button is clicked', async () => {
  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [{ maxOutput: '0x158323e942e36d8c', ratio: '0x5b16799fcb6114f7' }];
    }
  });

  const queryClient = new QueryClient();

  render(TanstackOrderQuote, {
    props: { orderHash: '0x123' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    const orderQuoteComponent = screen.getByTestId('orderQuoteComponent');
    expect(orderQuoteComponent).toHaveTextContent('1.550122181502135692');
  });

  mockIPC((cmd) => {
    if (cmd === 'batch_order_quotes') {
      return [{ maxOutput: '0x10ed6dd0a6e5d4cc', ratio: '0x5e68460c537594a0' }];
    }
  });

  screen.getByText('Refresh Quote').click();

  await waitFor(() => {
    const orderQuoteComponent = screen.getByTestId('orderQuoteComponent');

    expect(orderQuoteComponent).toHaveTextContent('Maximum output');
    expect(orderQuoteComponent).toHaveTextContent('1.219751817007977676');
    expect(orderQuoteComponent).toHaveTextContent('Price');
    expect(orderQuoteComponent).toHaveTextContent('6.802764255896900768');
  });
});
