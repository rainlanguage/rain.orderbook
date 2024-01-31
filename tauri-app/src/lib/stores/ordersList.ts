import { get } from 'svelte/store';
import type { Order } from '$lib/typeshare/ordersList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { usePaginatedCachedStore } from '$lib/stores/paginatedStore';

export const ordersList = usePaginatedCachedStore<Order>('ordersList', (page, pageSize = 10) => invoke("orders_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page, page_size: pageSize } }));