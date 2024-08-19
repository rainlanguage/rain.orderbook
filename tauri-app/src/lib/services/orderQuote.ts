import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, subgraphUrl } from '$lib/stores/settings';
import type { Order } from '$lib/typeshare/orderDetail';
import type { BatchOrderQuotesResponse } from '$lib/typeshare/orderQuote';

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
    orderbook: get(orderbookAddress),
  });
}
