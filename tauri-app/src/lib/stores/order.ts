import type { OrderDetailExtended, SgTrade } from '@rainlanguage/orderbook/js_api';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';
import { listStore } from '$lib/storesGeneric/listStore';

export const orderDetail = detailStore<OrderDetailExtended>(
  'orders.orderDetail',
  async (id: string) => {
    const url = await subgraphUrl.load();
    return invoke('order_detail', { id, subgraphArgs: { url } });
  },
);

export const useOrderTradesList = (orderId: string) =>
  listStore<SgTrade>(
    `orderTakesList-${orderId}`,
    async (page) => {
      const url = await subgraphUrl.load();
      return invoke('order_trades_list', {
        subgraphArgs: { url },
        orderId,
        paginationArgs: { page: page + 1, page_size: 10 },
      });
    },
    async (path) => {
      const url = await subgraphUrl.load();
      return invoke('order_trades_list_write_csv', { path, subgraphArgs: { url }, orderId });
    },
  );
