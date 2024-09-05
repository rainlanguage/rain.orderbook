import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from './constants';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Order } from '$lib/typeshare/subgraphTypes';

export type OrdersListArgs = {
  subgraphArgs: {
    url: string;
  };
  paginationArgs: {
    page: number;
    page_size: number;
  };
};

export const ordersList = async (
  url: string | undefined,
  owners: string[] = [],
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!url) {
    return [];
  }
  return await invoke<Order[]>('orders_list', {
    subgraphArgs: { url },
    filterArgs: {
      owners,
    },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  } as OrdersListArgs);
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('uses the orders_list command correctly', async () => {
    mockIPC((cmd) => {
      if (cmd === 'orders_list') {
        return [
          {
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
        ];
      }
    });

    // check for a result with no URL
    expect(await ordersList(undefined, [], 0)).toEqual([]);

    // check for a result with a URL
    expect(await ordersList('http://localhost:8000', [], 0)).toEqual([
      {
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
    ]);
  });
}
