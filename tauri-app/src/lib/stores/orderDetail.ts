import { get } from 'svelte/store';
import type { OrderDetailExtended } from '$lib/typeshare/orderDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';

export const orderDetail = detailStore<OrderDetailExtended>("orders.orderDetail", (id: string) => invoke("order_detail", {id, subgraphArgs: { url: get(subgraphUrl)} }));