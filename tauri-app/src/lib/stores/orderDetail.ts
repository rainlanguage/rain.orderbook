import { get } from 'svelte/store';
import type { Order } from '$lib/typeshare/orderDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';

export const orderDetail = detailStore<Order>("orders.orderDetail", (id: string) => invoke("order_detail", {id, subgraphArgs: { url: get(subgraphUrl)} }));