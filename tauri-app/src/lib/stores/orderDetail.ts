import type { OrderDetailExtended } from '$lib/typeshare/orderDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';

export const orderDetail = detailStore<OrderDetailExtended>("orders.orderDetail", async (id: string) => {
  const url = await subgraphUrl.load();
  return invoke("order_detail", {id, subgraphArgs: { url } });
});