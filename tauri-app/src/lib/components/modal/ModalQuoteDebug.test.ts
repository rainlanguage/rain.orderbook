import { render, screen, waitFor } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import { QueryClient } from '@tanstack/svelte-query';
import { formatEther } from 'viem';
import { mockQuoteDebug } from '$lib/queries/orderQuote';
import ModalQuoteDebug from './ModalQuoteDebug.svelte';

test('renders table with the correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'debug_order_quote') {
      return mockQuoteDebug;
    }
  });

  render(ModalQuoteDebug, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: {
      open: true,
      order: {
        id: '1',
        order_bytes: '0x123',
        order_hash: '0x123',
        owner: '0x123',
        outputs: [],
        inputs: [],
        active: true,
        add_events: [],
        timestamp_added: '123',
      },
      rpcUrl: 'https://rpc-url.com',
      inputIOIndex: 0,
      outputIOIndex: 0,
      orderbook: '0x123',
      pair: 'ETH/USDC',
    },
  });

  expect(await screen.findByTestId('modal-quote-debug-loading-message')).toBeInTheDocument();

  await waitFor(() => {
    expect(screen.queryByTestId('modal-quote-debug-rpc-url')).toHaveTextContent(
      'RPC: https://rpc-url.com',
    );
  });

  const stacks = await screen.findAllByTestId('debug-stack');
  expect(stacks).toHaveLength(3);
  const values = await screen.findAllByTestId('debug-value');
  expect(values).toHaveLength(3);
  const hexValues = await screen.findAllByTestId('debug-value-hex');
  for (let i = 0; i < 3; i++) {
    expect(stacks[i]).toHaveTextContent(mockQuoteDebug.column_names[i]);
    expect(values[i]).toHaveTextContent(formatEther(BigInt(mockQuoteDebug.rows[0][i])));
    expect(hexValues[i]).toHaveTextContent(mockQuoteDebug.rows[0][i]);
  }
});
