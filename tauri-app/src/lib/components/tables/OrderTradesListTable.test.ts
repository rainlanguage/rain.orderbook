import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Trade } from '$lib/typeshare/orderTakesList';
import { formatUnits } from 'viem';
import OrderTradesListTable from './OrderTradesListTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

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

const mockTakeOrdersList: Trade[] = [
  {
    id: '1',
    timestamp: '1632000000',
    trade_event: {
      sender: 'sender_address',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
      },
    },
    output_vault_balance_change: {
      amount: '100',
      vault: {
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '1',
        },
      },
    },
    order: {
      id: 'order_id',
      order_hash: 'order_hash',
      timestamp_added: '1632000000',
    },
    input_vault_balance_change: {
      vault: {
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '1',
        },
      },
      amount: '50',
    },
  },
  {
    id: '2',
    timestamp: '1632000000',
    trade_event: {
      sender: 'sender_address',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
      },
    },
    output_vault_balance_change: {
      amount: '100',
      vault: {
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '1',
        },
      },
    },
    order: {
      id: 'order_id',
      order_hash: 'order_hash',
      timestamp_added: '1632000000',
    },
    input_vault_balance_change: {
      vault: {
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '1',
        },
      },
      amount: '50',
    },
  },
];

test('renders table with correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'order_takes_list') {
      return mockTakeOrdersList;
    }
  });

  render(OrderTradesListTable, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: { id: '1' },
  });

  await waitFor(async () => {
    // get all the io ratios
    const rows = screen.getAllByTestId('io-ratio');

    // checking the io ratios
    for (let i = 0; i < mockTakeOrdersList.length; i++) {
      const inputDisplay = formatUnits(
        BigInt(mockTakeOrdersList[i].input_vault_balance_change.amount),
        Number(mockTakeOrdersList[i].input_vault_balance_change.vault.token.decimals),
      );
      const outputDisplay = formatUnits(
        BigInt(mockTakeOrdersList[i].output_vault_balance_change.amount),
        Number(mockTakeOrdersList[i].output_vault_balance_change.vault.token.decimals),
      );
      const expectedRatio = Number(inputDisplay) / Number(outputDisplay);
      expect(rows[i]).toHaveTextContent(expectedRatio.toString());
    }
  });
});
