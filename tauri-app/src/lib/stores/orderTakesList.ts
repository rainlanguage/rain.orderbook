import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';
import type { TakeOrderEntity } from '$lib/typeshare/orderTakesList';

export const useOrderTakesList = (orderId: string) =>  listStore<TakeOrderEntity>(
  `orderTakesList-${orderId}`,
  (page) => invoke("order_takes_list", {subgraphArgs: { url: get(subgraphUrl)}, orderId, paginationArgs: { page: page+1, page_size: 10 } }),
  (path) => invoke("order_takes_list_write_csv", {path, subgraphArgs: { url: get(subgraphUrl)}, orderId, paginationArgs: { page: 1, page_size: 1000 } })
);
