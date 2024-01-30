import { derived, get, writable } from 'svelte/store';
import type { Order } from '$lib/typeshare/ordersList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';

function useOrdersPagesStore() {
  const STORAGE_KEY = "orders.ordersList";

  const ordersPages = writable<{[page: number]: Array<Order>}>(localStorage.getItem(STORAGE_KEY) ? JSON.parse(localStorage.getItem(STORAGE_KEY) as string) : []);

  ordersPages.subscribe(value => {
    if(value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([]));
    }
  });

  return ordersPages;
}

export const ordersPages = useOrdersPagesStore();

export async function refetchOrdersPage(page: number = 1, pageSize: number = 10) {
  const res: Array<Order> = await invoke("orders_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page, page_size: pageSize } });
  ordersPages.update((val) => {
    val[page] = res;
    return val;
  });
}

export const ordersPage = derived(ordersPages, $ordersPages => (page: number) => $ordersPages[page] || []);
