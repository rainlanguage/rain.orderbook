import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Trade } from '$lib/typeshare/subgraphTypes';
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

vi.mock('$lib/services/modal', async () => {
  return {
    handleDepositGenericModal: vi.fn(),
    handleDepositModal: vi.fn(),
    handleWithdrawModal: vi.fn(),
  };
});

const mockTradeOrdersList: Trade[] = [
  {
    id: '1',
    timestamp: '1632000000',
    tradeEvent: {
      sender: 'sender_address',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
        blockNumber: '0',
      },
    },
    outputVaultBalanceChange: {
      amount: '-66463609853759340683',
      vault: {
        id: 'id',
        vault_id: 'vault-id',
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '18',
        },
      },
      id: '1',
      typename: 'Withdraw',
      newVaultBalance: '0',
      oldVaultBalance: '0',
      timestamp: '0',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
        blockNumber: '0',
      },
      orderbook: { id: '1' },
    },
    order: {
      id: 'order_id',
      orderHash: 'orderHash',
    },
    inputVaultBalanceChange: {
      vault: {
        id: 'id',
        vault_id: 'vault-id',
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '18',
        },
      },
      amount: '61112459033728404490',
      id: '1',
      typename: 'Withdraw',
      newVaultBalance: '0',
      oldVaultBalance: '0',
      timestamp: '0',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
        blockNumber: '0',
      },
      orderbook: { id: '1' },
    },
    orderbook: {
      id: '0x00',
    },
  },
  {
    id: '2',
    timestamp: '1632000000',
    tradeEvent: {
      sender: 'sender_address',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
        blockNumber: '0',
      },
    },
    outputVaultBalanceChange: {
      amount: '-100',
      vault: {
        id: 'id',
        vault_id: 'vault-id',
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '1',
        },
      },
      id: '1',
      typename: 'Withdraw',
      newVaultBalance: '0',
      oldVaultBalance: '0',
      timestamp: '0',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
        blockNumber: '0',
      },
      orderbook: { id: '1' },
    },
    order: {
      id: 'order_id',
      orderHash: 'orderHash',
    },
    inputVaultBalanceChange: {
      vault: {
        id: 'id',
        vault_id: 'vault-id',
        token: {
          id: 'output_token',
          address: 'output_token',
          name: 'output_token',
          symbol: 'output_token',
          decimals: '1',
        },
      },
      amount: '50',
      id: '1',
      typename: 'Withdraw',
      newVaultBalance: '0',
      oldVaultBalance: '0',
      timestamp: '0',
      transaction: {
        id: 'transaction_id',
        from: 'sender_address',
        timestamp: '1632000000',
        blockNumber: '0',
      },
      orderbook: { id: '1' },
    },
    orderbook: {
      id: '0x00',
    },
  },
];

test('renders table with correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'order_trades_list') {
      return mockTradeOrdersList;
    }
  });

  render(OrderTradesListTable, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: { id: '1' },
  });

  await waitFor(async () => {
    // get all the io ratios
    const rows = screen.getAllByTestId('io-ratio');

    // expect the first row to have the correct io ratio
    expect(rows[0]).toHaveTextContent('1.087562354790495301');
    expect(rows[0]).toHaveTextContent('0.919487508550842543');

    // expect the second row to have the correct io ratio
    expect(rows[1]).toHaveTextContent('2');
    expect(rows[1]).toHaveTextContent('0.5');
  });
});

test('renders a debug button for each trade', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'order_trades_list') {
      return mockTradeOrdersList;
    }
  });

  render(OrderTradesListTable, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: { id: '1' },
  });

  await waitFor(async () => {
    const buttons = screen.getAllByTestId('debug-trade-button');
    expect(buttons).toHaveLength(mockTradeOrdersList.length);
  });
});
