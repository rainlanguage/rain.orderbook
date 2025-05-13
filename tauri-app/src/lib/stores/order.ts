import type { OrderDetailExtended, SgTrade } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';
import { subgraph } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';
import { listStore } from '$lib/storesGeneric/listStore';

export const orderDetail = detailStore<OrderDetailExtended>(
  'orders.orderDetail',
  async (id: string) => {
    const loadedSubgraph = await subgraph.load();
    return invoke('order_detail', { id, subgraphArgs: { url: loadedSubgraph?.url } });
  },
);

export const useOrderTradesList = (orderId: string) =>
  listStore<SgTrade>(
    `orderTakesList-${orderId}`,
    async (page) => {
      const loadedSubgraph = await subgraph.load();
      return invoke('order_trades_list', {
        subgraphArgs: { url: loadedSubgraph?.url },
        orderId,
        paginationArgs: { page: page + 1, page_size: 10 },
      });
    },
    async (path) => {
      const loadedSubgraph = await subgraph.load();
      return invoke('order_trades_list_write_csv', {
        path,
        subgraphArgs: { url: loadedSubgraph?.url },
        orderId,
      });
    },
  );
