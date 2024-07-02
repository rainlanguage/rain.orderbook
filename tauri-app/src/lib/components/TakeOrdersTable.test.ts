import { render, screen } from '@testing-library/svelte';
import { test } from 'vitest';
import TakeOrdersTable from './TakeOrdersTable.svelte';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import { useOrderTakesList } from '$lib/stores/order';
import type { Trade } from '$lib/typeshare/orderTakesList';
import { formatUnits } from 'viem';

// a mock repsonse to the subgraph query above, but in rust format
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
  mockIPC((cmd) => {
    if (cmd === 'order_takes_list') {
      return mockTakeOrdersList;
    }
  });

  const orderTakesList = useOrderTakesList('1');

  render(TakeOrdersTable, { orderTakesList });
  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 0));

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
