import type { TokenVault } from '$lib/typeshare/vaultsList';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';


export const vaultsList = listStore<TokenVault>(
  'vaultsList',
  async (page) => {
    const url = await subgraphUrl.load();
    return invoke("vaults_list", {subgraphArgs: { url }, paginationArgs: { page: page+1, page_size: 10 } });
  },
  async (path) => {
    const url = await subgraphUrl.load();
    return invoke("vaults_list_write_csv", {path, subgraphArgs: { url }});
  },
);
