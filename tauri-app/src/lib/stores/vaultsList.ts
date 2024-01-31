import { get } from 'svelte/store';
import type { TokenVault } from '$lib/typeshare/vaultsList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { usePaginatedCachedStore } from './paginatedStore';


export const vaultsList = usePaginatedCachedStore<TokenVault>(
  'vaultsList',
  (page, pageSize = 10) => invoke("vaults_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page, page_size: pageSize } }),
  (path) => invoke("vaults_list_write_csv", {path, subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page: 1, page_size: 1000 } })
);
