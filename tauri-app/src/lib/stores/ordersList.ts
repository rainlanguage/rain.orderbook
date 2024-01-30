import { get, writable } from 'svelte/store';
import type { Order } from '$lib/typeshare/ordersList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';

function useOrdersListStore() {
  const STORAGE_KEY = "orders.ordersList";

  const { subscribe, set } = writable<Array<Order>>(localStorage.getItem(STORAGE_KEY) ? JSON.parse(localStorage.getItem(STORAGE_KEY) as string) : []);

  subscribe(value => {
    if(value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([]));
    }
  });

  async function refetch() {
    const res: Array<Order> = await invoke("orders_list", {subgraphArgs: { url: get(subgraphUrl)} });
    set(res);
  }

  return {
    subscribe,
    refetch
  }
}

export const ordersList = useOrdersListStore();