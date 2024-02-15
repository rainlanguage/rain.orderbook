import { get } from 'svelte/store';
import type { OrderClear } from '$lib/typeshare/orderClearsList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';

export const orderClearsList = listStore<OrderClear>(
  'orderClearsList',
  (page) => invoke("order_clears_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page: page+1, page_size: 10 } }),
  (path) => invoke("order_clears_list_write_csv", { path, subgraphArgs: { url: get(subgraphUrl)} })
);