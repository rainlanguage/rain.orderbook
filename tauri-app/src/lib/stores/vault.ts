import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';
import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';
import { detailStore } from '$lib/storesGeneric/detailStore';
import type { TokenVault } from '$lib/typeshare/vaultsList';

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

export const vaultDetail = detailStore<TokenVault>("vaults.vaultsDetail", async (id: string) => {
  const url = await subgraphUrl.load();
  return invoke("vault_detail", {id, subgraphArgs: { url } });
});

export const useVaultBalanceChangesList = (id: string) =>  listStore<VaultBalanceChange>(
  `vaultBalanceChangesList-${id}`,
  async (page) => {
    const url = await subgraphUrl.load();
    return invoke("vault_balance_changes_list", {subgraphArgs: { url }, id, paginationArgs: { page: page+1, page_size: 10 } })
  },
  async (path) => {
    const url = await subgraphUrl.load();
    return invoke("vault_balance_changes_list_write_csv", {path, subgraphArgs: { url }, id})
  },
);