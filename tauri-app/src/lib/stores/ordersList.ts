import { get, writable } from 'svelte/store';
import type { Order as OrdersListItem } from '$lib/typeshare/orders';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';

function useOrdersListStore() {
  const STORAGE_KEY = "orders.ordersList";

  const { subscribe, set } = writable<Array<OrdersListItem>>(localStorage.getItem(STORAGE_KEY) ? JSON.parse(localStorage.getItem(STORAGE_KEY) as string) : []);

  subscribe(value => {
    if(value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([]));
    }
  });

  async function refetch() {
    const res: Array<OrdersListItem> = await invoke("orders_list", {subgraphArgs: { url: get(subgraphUrl)} });
    set(res);
  }

  return {
    subscribe,
    refetch
  }
}

export const ordersList = useOrdersListStore();