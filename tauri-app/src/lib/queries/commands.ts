import type { Vault } from '$lib/typeshare/vaultsList';
import type { Vault as VaultDetail } from '$lib/typeshare/vaultDetail';
import { invoke } from '@tauri-apps/api';
import { DEFAULT_PAGE_SIZE } from './constants';
import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';

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

export const vaultBalanceChangesList = async (
  id: string,
  url: string,
  pageParam: number,
  pageSize: number = DEFAULT_PAGE_SIZE,
) => {
  return await invoke<VaultBalanceChange[]>('vault_balance_changes_list', {
    subgraphArgs: { url },
    id,
    paginationArgs: { page: pageParam + 1, page_size: pageSize },
  });
};

export const vaultDetail = async (id: string, url: string) => {
  return await invoke<VaultDetail>('vault_detail', { id, subgraphArgs: { url } });
};
