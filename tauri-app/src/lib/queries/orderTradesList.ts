import type { Trade, VaultVolume } from '$lib/typeshare/subgraphTypes';
import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from '@rainlanguage/ui-components';
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
    pageSize: number;
  };
  startTimestamp?: number;
  endTimestamp?: number;
};

export const orderTradesList = async (
  id: string,
  url: string | undefined,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
  startTimestamp?: number,
  endTimestamp?: number,
) => {
  if (!url) {
    return [];
  }
  return await invoke<Trade[]>('order_trades_list', {
    orderId: id,
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, pageSize },
    startTimestamp,
    endTimestamp,
  } as OrderTradesListArgs);
};

export const orderTradesListForChart = async (
  id: string,
  url: string | undefined,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
  startTimestamp?: number,
  endTimestamp?: number,
) => {
  if (!url) {
    return [];
  }
  const data = await invoke<Trade[]>('order_trades_list', {
    orderId: id,
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, pageSize },
    startTimestamp,
    endTimestamp,
  } as OrderTradesListArgs);
  return prepareHistoricalOrderChartData(data, get(colorTheme));
};

export const orderVaultsVolume = async (
  id: string,
  url: string | undefined,
  startTimestamp?: number,
  endTimestamp?: number,
) => {
  if (!url) {
    return [];
  }
  return await invoke<VaultVolume[]>('order_vaults_volume', {
    orderId: id,
    subgraphArgs: { url },
    startTimestamp,
    endTimestamp,
  } as OrderTradesListArgs);
};

export const orderTradesCount = async (
  id: string,
  url: string | undefined,
  startTimestamp?: number,
  endTimestamp?: number,
) => {
  if (!url) {
    return [];
  }
  return await invoke<number>('order_trades_count', {
    orderId: id,
    subgraphArgs: { url },
    startTimestamp,
    endTimestamp,
  } as OrderTradesListArgs);
};
