import { render, screen } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/__tests__/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import { QueryClient } from '@tanstack/svelte-query';
import { formatEther } from 'viem';
import { mockQuoteDebug } from '$lib/queries/orderQuote';
import ModalQuoteDebug from './ModalQuoteDebug.svelte';
import type { NetworkCfg, RaindexOrder, SgOrder } from '@rainlanguage/orderbook';

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
  return {
    ...(await importOriginal()),
    useRaindexClient: vi.fn(() => ({
      getNetworkByChainId: vi.fn().mockReturnValue({ value: {} as NetworkCfg }),
      getAllNetworks: vi.fn().mockReturnValue({ value: new Map() }),
    })),
  };
});

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
        convertToSgOrder: () => {
          return {
            value: {
              id: '1',
              orderbook: {
                id: '0x00',
                orderBytes: '0x123',
                orderHash: '0x123',
                owner: '0x123',
                outputs: [],
                inputs: [],
                active: true,
                timestampAdded: '123',
              },
            } as unknown as SgOrder,
          };
        },
      } as unknown as RaindexOrder,
      inputIOIndex: 0,
      outputIOIndex: 0,
      pair: 'ETH/USDC',
      blockNumber: BigInt(123),
    },
  });

  expect(await screen.findByTestId('modal-quote-debug-loading-message')).toBeInTheDocument();

  const stacks = await screen.findAllByTestId('debug-stack');
  expect(stacks).toHaveLength(3);
  const values = await screen.findAllByTestId('debug-value');
  expect(values).toHaveLength(3);
  const hexValues = await screen.findAllByTestId('debug-value-hex');
  for (let i = 0; i < 3; i++) {
    expect(stacks[i]).toHaveTextContent(mockQuoteDebug[0].columnNames[i]);
    expect(values[i]).toHaveTextContent(formatEther(BigInt(mockQuoteDebug[0].rows[0][i])));
    expect(hexValues[i]).toHaveTextContent(mockQuoteDebug[0].rows[0][i]);
  }
  const partialError = await screen.findAllByTestId('modal-quote-debug-error-partial');
  expect(partialError[0]).toHaveTextContent(mockQuoteDebug[1]!);
});
