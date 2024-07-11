import type { Vault } from '$lib/typeshare/vaultsList';
import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from './constants';

export const vaultBalanceList = async (
  url: string,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  return await invoke<Vault[]>('vaults_list', {
    subgraphArgs: { url },
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  });
};
