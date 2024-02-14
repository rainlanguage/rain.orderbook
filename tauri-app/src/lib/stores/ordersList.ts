import { get } from 'svelte/store';
import type { Order } from '$lib/typeshare/ordersList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';

export const ordersList = listStore<Order>(
  'ordersList',
  (page) => invoke("orders_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page: page+1, page_size: 10 } }),
  (path) => invoke("orders_list_write_csv", { path, subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page: 1, page_size: 1000 } })
);