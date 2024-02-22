import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';
import type { TakeOrderEntity } from '$lib/typeshare/orderTakesList';

export const useOrderTakesList = (orderId: string) =>  listStore<TakeOrderEntity>(
  `orderTakesList-${orderId}`,
  async (page) => {
    const url = await subgraphUrl.load();
    return invoke("order_takes_list", {subgraphArgs: { url }, orderId, paginationArgs: { page: page+1, page_size: 10 } });
  },
  async (path) => {
    const url = await subgraphUrl.load();
    return invoke("order_takes_list_write_csv", {path, subgraphArgs: { url }, orderId});
  },
);
