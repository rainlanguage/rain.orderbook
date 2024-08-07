import type { Trade } from '$lib/typeshare/orderTakesList';
import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from './constants';
import { prepareHistoricalOrderChartData } from '$lib/services/historicalOrderCharts';
import { colorTheme } from '$lib/stores/darkMode';
import { get } from 'svelte/store';

export type OrderTradesListArgs = {
  orderId: string;
  subgraphArgs: {
    url: string;
  };
  paginationArgs: {
    page: number;
    page_size: number;
  };
};

export const orderTradesList = async (
  id: string,
  url: string | undefined,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!url) {
    return [];
  }
  return await invoke<Trade[]>('order_takes_list', {
    orderId: id,
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  } as OrderTradesListArgs);
};

export const orderTradesListForChart = async (
  id: string,
  url: string | undefined,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  if (!url) {
    return [];
  }
  const data = await invoke<Trade[]>('order_takes_list', {
    orderId: id,
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  } as OrderTradesListArgs);
  return prepareHistoricalOrderChartData(data, get(colorTheme));
};
