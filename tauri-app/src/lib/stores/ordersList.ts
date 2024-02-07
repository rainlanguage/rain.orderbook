import { get } from 'svelte/store';
import type { Order } from '$lib/typeshare/ordersList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { usePaginatedCachedStore } from '$lib/storesGeneric/paginatedStore';

export const ordersList = usePaginatedCachedStore<Order>(
  'ordersList',
  (page) => invoke("orders_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page, page_size: 10 } }),
  (path) => invoke("orders_list_write_csv", { path, subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page: 1, page_size: 1000 } })
);