import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, subgraphUrl } from '$lib/stores/settings';
import type { Order } from '$lib/typeshare/orderDetail';
import type { BatchOrderQuotesResponse } from '$lib/typeshare/orderQuote';
import type { Hex } from 'viem';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { RainEvalResultsTable } from '$lib/typeshare/config';

const formatOrder = (order: Order) => ({
  ...order,
  orderBytes: order.order_bytes,
  orderHash: order.order_hash,
  outputs: order.outputs.map((output) => ({
    ...output,
    vaultId: output.vault_id,
  })),
  inputs: order.inputs.map((input) => ({
    ...input,
    vaultId: input.vault_id,
  })),
  addEvents: order.add_events.map((event) => ({
    ...event,
    transaction: {
      ...event.transaction,
      blockNumber: event.transaction.block_number,
    },
  })),
  timestampAdded: order.timestamp_added,
});

export async function batchOrderQuotes(orders: Order[]): Promise<BatchOrderQuotesResponse[]> {
  const formattedOrders = orders.map((order) => ({
    ...order,
    orderBytes: order.order_bytes,
    orderHash: order.order_hash,
    outputs: order.outputs.map((output) => ({
      ...output,
      vaultId: output.vault_id,
    })),
    inputs: order.inputs.map((input) => ({
      ...input,
      vaultId: input.vault_id,
    })),
    addEvents: order.add_events.map((event) => ({
      ...event,
      transaction: {
        ...event.transaction,
        blockNumber: event.transaction.block_number,
      },
    })),
    timestampAdded: order.timestamp_added,
  }));
  return invoke('batch_order_quotes', {
    orders: formattedOrders,
    subgraphUrl: get(subgraphUrl),
    rpcUrl: get(rpcUrl),
  });
}

export async function debugOrderQuote(
  order: Order,
  inputIOIndex: number,
  outputIOIndex: number,
  orderbook: Hex,
  rpcUrl: string,
) {
  return await invoke<RainEvalResultsTable>('debug_order_quote', {
    order: formatOrder(order),
    inputIoIndex: inputIOIndex,
    outputIoIndex: outputIOIndex,
    orderbook,
    rpcUrl,
  });
}

export const mockQuoteDebug: RainEvalResultsTable = {
  column_names: ['1', '2', '3'],
  rows: [['0x01', '0x02', '0x03']],
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the trade_debug command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'debug_order_quote') {
        return mockQuoteDebug;
      }
    });

    const result = await debugOrderQuote(
      {
        id: '1',
        orderbook: { id: '0x00' },
        order_bytes: '0x123',
        order_hash: '0x123',
        owner: '0x123',
        outputs: [],
        inputs: [],
        active: true,
        add_events: [],
        timestamp_added: '123',
      },
      0,
      0,
      '0x123',
      'https://rpc-url.com',
    );
    expect(result).toEqual(mockQuoteDebug);
  });
}
