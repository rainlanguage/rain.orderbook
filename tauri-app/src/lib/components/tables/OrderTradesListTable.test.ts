import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Trade } from '$lib/typeshare/subgraphTypes';
import { formatUnits } from 'viem';
import OrderTradesListTable from './OrderTradesListTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { writable } = await import('svelte/store');
  const { mockSettingsStore } = await import('@rainlanguage/ui-components');

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

    // checking the io ratios
    for (let i = 0; i < mockTradeOrdersList.length; i++) {
      const inputDisplay = formatUnits(
        BigInt(mockTradeOrdersList[i].inputVaultBalanceChange.amount),
        Number(mockTradeOrdersList[i].inputVaultBalanceChange.vault.token.decimals),
      );
      const outputDisplay = formatUnits(
        BigInt(mockTradeOrdersList[i].outputVaultBalanceChange.amount),
        Number(mockTradeOrdersList[i].outputVaultBalanceChange.vault.token.decimals),
      );
      const ioRatio = Number(inputDisplay) / (Number(outputDisplay) * -1);
      const oiRatio = (Number(outputDisplay) * -1) / Number(inputDisplay);
      expect(rows[i]).toHaveTextContent(ioRatio.toString());
      expect(rows[i]).toHaveTextContent(oiRatio.toString());
    }
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
