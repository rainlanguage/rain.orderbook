import type { VaultVolume } from '$lib/typeshare/subgraphTypes';
import { invoke } from '@tauri-apps/api';

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
