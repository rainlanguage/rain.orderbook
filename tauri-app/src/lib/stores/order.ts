import type { OrderDetailExtended } from '$lib/typeshare/orderDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';
import { listStore } from '$lib/storesGeneric/listStore';
import type { Trade } from '$lib/typeshare/orderTakesList';

export const orderDetail = detailStore<OrderDetailExtended>(
  'orders.orderDetail',
  async (id: string) => {
    const url = await subgraphUrl.load();
    return invoke('order_detail', { id, subgraphArgs: { url } });
  },
);

export const useOrderTakesList = (orderId: string) =>
  listStore<Trade>(
    `orderTakesList-${orderId}`,
    async (page) => {
      const url = await subgraphUrl.load();
      return invoke('order_takes_list', {
        subgraphArgs: { url },
        orderId,
        paginationArgs: { page: page + 1, page_size: 10 },
      });
    },
    async (path) => {
      const url = await subgraphUrl.load();
      return invoke('order_takes_list_write_csv', { path, subgraphArgs: { url }, orderId });
    },
  );
