import { get } from 'svelte/store';
import type { TokenVault } from '$lib/typeshare/vaultsList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';


export const vaultsList = listStore<TokenVault>(
  'vaultsList',
  (page) => invoke("vaults_list", {subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page, page_size: 10 } }),
  (path) => invoke("vaults_list_write_csv", {path, subgraphArgs: { url: get(subgraphUrl)}, paginationArgs: { page: 1, page_size: 1000 } })
);
