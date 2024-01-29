import { get, writable } from 'svelte/store';
import type { Order as OrderDetail } from '$lib/typeshare/order';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';

function useOrderDetailStore() {
  const STORAGE_KEY = "orders.orderDetail";

  const { subscribe, update } = writable<{[id: string]: OrderDetail}>(localStorage.getItem(STORAGE_KEY) ? JSON.parse(localStorage.getItem(STORAGE_KEY) as string) : {});

  subscribe(value => {
    if(value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({}));
    }
  });

  async function refetch(id: string) {
    const res: OrderDetail = await invoke("order_detail", {id, subgraphArgs: { url: get(subgraphUrl)} });
    update((value) => {
      return {... value, [id]: res};
    });
  }

  return {
    subscribe,
    refetch
  }
}

export const orderDetail = useOrderDetailStore();