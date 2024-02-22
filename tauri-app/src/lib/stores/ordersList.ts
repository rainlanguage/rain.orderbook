import type { Order } from '$lib/typeshare/ordersList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';

export const ordersList = listStore<Order>(
  'ordersList',
  async (page) => {
    const url = await subgraphUrl.load();
    return invoke("orders_list", {subgraphArgs: { url }, paginationArgs: { page: page+1, page_size: 10 } });
  },
  async (path) => {
    const url = await subgraphUrl.load();
    return invoke("orders_list_write_csv", { path, subgraphArgs: { url } });
  }
);