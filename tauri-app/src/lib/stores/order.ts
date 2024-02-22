import type { OrderDetailExtended } from '$lib/typeshare/orderDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';
import type { Order } from '$lib/typeshare/ordersList';
import { listStore } from '$lib/storesGeneric/listStore';
import type { TakeOrderEntity } from '$lib/typeshare/orderTakesList';

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

export const orderDetail = detailStore<OrderDetailExtended>("orders.orderDetail", async (id: string) => {
  const url = await subgraphUrl.load();
  return invoke("order_detail", {id, subgraphArgs: { url } });
});

export const useOrderTakesList = (orderId: string) =>  listStore<TakeOrderEntity>(
  `orderTakesList-${orderId}`,
  async (page) => {
    const url = await subgraphUrl.load();
    return invoke("order_takes_list", {subgraphArgs: { url }, orderId, paginationArgs: { page: page+1, page_size: 10 } });
  },
  async (path) => {
    const url = await subgraphUrl.load();
    return invoke("order_takes_list_write_csv", {path, subgraphArgs: { url }, orderId});
  },
);
