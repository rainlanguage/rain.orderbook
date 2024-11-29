import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from '@rainlanguage/ui-components';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { OrderWithSubgraphName } from '$lib/typeshare/subgraphTypes';

export type OrdersListArgs = {
  multiSubgraphArgs: {
    url: string;
    name: string;
  }[];
  filterArgs: {
    owners: string[];
    active: boolean | undefined;
    orderHash: string | undefined;
  };
  paginationArgs: {
    page: number;
    pageSize: number;
  };
};

export const ordersList = async (
  activeSubgraphs: Record<string, string>,
  owners: string[] = [],
  active: boolean | undefined = undefined,
  orderHash: string = '',
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!Object.keys(activeSubgraphs).length) {
    return [];
  }
  return await invoke<OrderWithSubgraphName[]>('orders_list', {
    multiSubgraphArgs: Object.entries(activeSubgraphs).map(([name, url]) => ({
      name,
      url,
    })),
    filterArgs: {
      owners,
      active,
      orderHash: orderHash || undefined,
    },
    paginationArgs: { page: pageParam + 1, pageSize },
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
    expect(await ordersList({}, [], undefined, undefined, 0)).toEqual([]);

    // check for a result with a URL
    expect(await ordersList({ network: 'url' }, [], undefined, undefined, 0)).toEqual([
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
